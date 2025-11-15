//! # Access Control Pallet
//!
//! ## Overview
//!
//! The Access Control pallet enforces consent-based access to health records.
//! It provides functionality for:
//! - Consent verification (queries IdentityConsent Chain via XCM)
//! - Access request management
//! - Permission caching for performance
//! - Access denial logging
//!
//! ## Architecture Reference
//! See parachain.md Section: "HealthData Chain - Access Control"

#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::{pallet_prelude::*, traits::Time};
    use frame_system::pallet_prelude::*;
    use sp_std::prelude::*;
    use sp_core::H256;

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    /// Access request status
    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub enum AccessStatus {
        /// Access request pending consent verification
        Pending,
        /// Access granted
        Granted,
        /// Access denied
        Denied,
        /// Consent expired
        Expired,
    }

    /// Access request
    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    #[scale_info(skip_type_params(T))]
    pub struct AccessRequest<T: Config> {
        /// Request ID
        pub request_id: H256,
        /// Record ID being accessed
        pub record_id: H256,
        /// Requester account
        pub requester: T::AccountId,
        /// Patient (record owner)
        pub patient: T::AccountId,
        /// Consent ID (from IdentityConsent Chain)
        pub consent_id: Option<H256>,
        /// Request status
        pub status: AccessStatus,
        /// Request timestamp
        pub requested_at: u64,
        /// Response timestamp
        pub responded_at: Option<u64>,
    }

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// The overarching event type
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// Time provider for timestamps
        type TimeProvider: Time;
    }

    /// Storage for access requests by request_id
    #[pallet::storage]
    #[pallet::getter(fn access_requests)]
    pub type AccessRequests<T: Config> = StorageMap<_, Blake2_128Concat, H256, AccessRequest<T>>;

    /// Storage for active access grants (record_id -> requester -> expires_at)
    #[pallet::storage]
    #[pallet::getter(fn access_grants)]
    pub type AccessGrants<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        H256, // record_id
        Blake2_128Concat,
        T::AccountId, // requester
        u64, // expires_at
    >;

    /// Request counter
    #[pallet::storage]
    #[pallet::getter(fn request_count)]
    pub type RequestCount<T: Config> = StorageValue<_, u64, ValueQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Access requested [request_id, record_id, requester]
        AccessRequested {
            request_id: H256,
            record_id: H256,
            requester: T::AccountId,
        },
        /// Access granted [request_id, record_id, requester]
        AccessGranted {
            request_id: H256,
            record_id: H256,
            requester: T::AccountId,
        },
        /// Access denied [request_id, record_id, requester]
        AccessDenied {
            request_id: H256,
            record_id: H256,
            requester: T::AccountId,
        },
        /// Access revoked [record_id, requester]
        AccessRevoked {
            record_id: H256,
            requester: T::AccountId,
        },
    }

    #[pallet::error]
    pub enum Error<T> {
        /// Request not found
        RequestNotFound,
        /// Not authorized
        NotAuthorized,
        /// Access already granted
        AccessAlreadyGranted,
        /// No active consent found
        NoActiveConsent,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Request access to a health record
        #[pallet::call_index(0)]
        #[pallet::weight(10_000)]
        pub fn request_access(
            origin: OriginFor<T>,
            record_id: H256,
            patient: T::AccountId,
            consent_id: H256,
        ) -> DispatchResult {
            let requester = ensure_signed(origin)?;

            let now = T::TimeProvider::now();

            // Generate request ID
            let count = RequestCount::<T>::get();
            let request_id = Self::generate_request_id(&requester, count);
            RequestCount::<T>::put(count.saturating_add(1));

            let request = AccessRequest {
                request_id,
                record_id,
                requester: requester.clone(),
                patient,
                consent_id: Some(consent_id),
                status: AccessStatus::Pending,
                requested_at: now,
                responded_at: None,
            };

            AccessRequests::<T>::insert(request_id, request);

            Self::deposit_event(Event::AccessRequested {
                request_id,
                record_id,
                requester,
            });

            Ok(())
        }

        /// Grant access to a record (called after consent verification)
        #[pallet::call_index(1)]
        #[pallet::weight(10_000)]
        pub fn grant_access(
            origin: OriginFor<T>,
            request_id: H256,
            expires_at: u64,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            AccessRequests::<T>::try_mutate(request_id, |maybe_request| -> DispatchResult {
                let request = maybe_request.as_mut().ok_or(Error::<T>::RequestNotFound)?;

                // Only patient can grant access
                ensure!(request.patient == who, Error::<T>::NotAuthorized);

                let now = T::TimeProvider::now();
                request.status = AccessStatus::Granted;
                request.responded_at = Some(now);

                // Store access grant
                AccessGrants::<T>::insert(&request.record_id, &request.requester, expires_at);

                Self::deposit_event(Event::AccessGranted {
                    request_id,
                    record_id: request.record_id,
                    requester: request.requester.clone(),
                });

                Ok(())
            })
        }

        /// Deny access request
        #[pallet::call_index(2)]
        #[pallet::weight(10_000)]
        pub fn deny_access(origin: OriginFor<T>, request_id: H256) -> DispatchResult {
            let who = ensure_signed(origin)?;

            AccessRequests::<T>::try_mutate(request_id, |maybe_request| -> DispatchResult {
                let request = maybe_request.as_mut().ok_or(Error::<T>::RequestNotFound)?;

                ensure!(request.patient == who, Error::<T>::NotAuthorized);

                let now = T::TimeProvider::now();
                request.status = AccessStatus::Denied;
                request.responded_at = Some(now);

                Self::deposit_event(Event::AccessDenied {
                    request_id,
                    record_id: request.record_id,
                    requester: request.requester.clone(),
                });

                Ok(())
            })
        }

        /// Revoke access to a record
        #[pallet::call_index(3)]
        #[pallet::weight(10_000)]
        pub fn revoke_access(
            origin: OriginFor<T>,
            record_id: H256,
            requester: T::AccountId,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // Remove access grant
            AccessGrants::<T>::remove(&record_id, &requester);

            Self::deposit_event(Event::AccessRevoked { record_id, requester });

            Ok(())
        }
    }

    // Helper functions
    impl<T: Config> Pallet<T> {
        /// Generate unique request ID
        fn generate_request_id(requester: &T::AccountId, nonce: u64) -> H256 {
            use sp_runtime::traits::Hash;
            let mut data = requester.encode();
            data.extend_from_slice(&nonce.encode());
            T::Hashing::hash(&data)
        }

        /// Check if access is granted
        pub fn has_access(record_id: &H256, requester: &T::AccountId, now: u64) -> bool {
            if let Some(expires_at) = AccessGrants::<T>::get(record_id, requester) {
                expires_at > now
            } else {
                false
            }
        }
    }
}
