//! # Identity Registry Pallet
//!
//! ## Overview
//!
//! The Identity Registry pallet manages user identities for the Patient X medical data marketplace.
//! It provides functionality for:
//! - User registration with role-based identity (Patient, Researcher, Institution, Auditor)
//! - DID (Decentralized Identifier) management
//! - Identity verification and attestation
//! - Profile management
//!
//! ## Architecture Reference
//! See parachain.md Section: "IdentityConsent Chain - Identity Registry"

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
    use sp_runtime::traits::UniqueSaturatedInto;

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    /// User role types in the Patient X ecosystem
    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub enum UserRole {
        /// Patient - owns medical data
        Patient,
        /// Researcher - consumes data for research
        Researcher,
        /// Institution - healthcare provider or research institution
        Institution,
        /// Auditor - compliance and oversight
        Auditor,
    }

    /// Identity verification status
    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub enum VerificationStatus {
        /// Not verified
        Unverified,
        /// Pending verification
        Pending,
        /// Verified by authority
        Verified,
        /// Verification rejected
        Rejected,
    }

    /// User identity information
    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    #[scale_info(skip_type_params(T))]
    pub struct Identity<T: Config> {
        /// Unique DID (Decentralized Identifier)
        pub did: BoundedVec<u8, ConstU32<100>>,
        /// User role
        pub role: UserRole,
        /// Display name
        pub name: BoundedVec<u8, ConstU32<64>>,
        /// Email hash for privacy
        pub email_hash: H256,
        /// Verification status
        pub verification_status: VerificationStatus,
        /// Registration timestamp
        pub registered_at: u64,
        /// Last update timestamp
        pub updated_at: u64,
        /// Account owner
        pub owner: T::AccountId,
        /// Active status
        pub active: bool,
    }

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// The overarching event type
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// Time provider for timestamps
        type TimeProvider: Time;

        /// Maximum number of identities per account
        #[pallet::constant]
        type MaxIdentitiesPerAccount: Get<u32>;
    }

    /// Storage for user identities by account ID
    #[pallet::storage]
    #[pallet::getter(fn identities)]
    pub type Identities<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, Identity<T>>;

    /// Storage for DID to AccountId mapping
    #[pallet::storage]
    #[pallet::getter(fn did_to_account)]
    pub type DidToAccount<T: Config> = StorageMap<_, Blake2_128Concat, BoundedVec<u8, ConstU32<100>>, T::AccountId>;

    /// Storage for verification requests
    #[pallet::storage]
    #[pallet::getter(fn verification_queue)]
    pub type VerificationQueue<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, u64>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Identity registered [account_id, did, role]
        IdentityRegistered {
            account: T::AccountId,
            did: BoundedVec<u8, ConstU32<100>>,
            role: UserRole
        },
        /// Identity updated [account_id]
        IdentityUpdated { account: T::AccountId },
        /// Identity deactivated [account_id]
        IdentityDeactivated { account: T::AccountId },
        /// Verification requested [account_id]
        VerificationRequested { account: T::AccountId },
        /// Identity verified [account_id, verifier]
        IdentityVerified {
            account: T::AccountId,
            verifier: T::AccountId
        },
        /// Verification rejected [account_id, reason]
        VerificationRejected {
            account: T::AccountId,
            reason: BoundedVec<u8, ConstU32<128>>
        },
    }

    #[pallet::error]
    pub enum Error<T> {
        /// Identity already exists for this account
        IdentityAlreadyExists,
        /// Identity not found
        IdentityNotFound,
        /// DID already registered
        DIDAlreadyExists,
        /// Not authorized to perform this action
        NotAuthorized,
        /// Identity is not active
        IdentityNotActive,
        /// Verification already pending
        VerificationAlreadyPending,
        /// Invalid DID format
        InvalidDID,
        /// Invalid name
        InvalidName,
        /// Maximum identities per account reached
        MaxIdentitiesReached,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Register a new identity
        ///
        /// Parameters:
        /// - `origin`: The account registering the identity
        /// - `did`: Decentralized identifier (unique)
        /// - `role`: User role in the ecosystem
        /// - `name`: Display name
        /// - `email_hash`: Hashed email for privacy
        #[pallet::call_index(0)]
        #[pallet::weight(10_000)]
        pub fn register_identity(
            origin: OriginFor<T>,
            did: BoundedVec<u8, ConstU32<100>>,
            role: UserRole,
            name: BoundedVec<u8, ConstU32<64>>,
            email_hash: H256,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // Ensure identity doesn't already exist
            ensure!(!Identities::<T>::contains_key(&who), Error::<T>::IdentityAlreadyExists);

            // Ensure DID is unique
            ensure!(!DidToAccount::<T>::contains_key(&did), Error::<T>::DIDAlreadyExists);

            // Validate DID format (basic check)
            ensure!(did.len() >= 10, Error::<T>::InvalidDID);

            // Validate name
            ensure!(!name.is_empty(), Error::<T>::InvalidName);

            let now: u64 = T::TimeProvider::now().unique_saturated_into();

            let identity = Identity {
                did: did.clone(),
                role: role.clone(),
                name,
                email_hash,
                verification_status: VerificationStatus::Unverified,
                registered_at: now,
                updated_at: now,
                owner: who.clone(),
                active: true,
            };

            // Store identity
            Identities::<T>::insert(&who, identity);
            DidToAccount::<T>::insert(&did, &who);

            Self::deposit_event(Event::IdentityRegistered {
                account: who,
                did,
                role,
            });

            Ok(())
        }

        /// Update identity information
        #[pallet::call_index(1)]
        #[pallet::weight(10_000)]
        pub fn update_identity(
            origin: OriginFor<T>,
            name: Option<BoundedVec<u8, ConstU32<64>>>,
            email_hash: Option<H256>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            Identities::<T>::try_mutate(&who, |maybe_identity| -> DispatchResult {
                let identity = maybe_identity.as_mut().ok_or(Error::<T>::IdentityNotFound)?;

                ensure!(identity.active, Error::<T>::IdentityNotActive);

                if let Some(new_name) = name {
                    ensure!(!new_name.is_empty(), Error::<T>::InvalidName);
                    identity.name = new_name;
                }

                if let Some(new_email_hash) = email_hash {
                    identity.email_hash = new_email_hash;
                }

                identity.updated_at = T::TimeProvider::now().unique_saturated_into();

                Self::deposit_event(Event::IdentityUpdated { account: who.clone() });

                Ok(())
            })
        }

        /// Request identity verification
        #[pallet::call_index(2)]
        #[pallet::weight(10_000)]
        pub fn request_verification(origin: OriginFor<T>) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let identity = Identities::<T>::get(&who).ok_or(Error::<T>::IdentityNotFound)?;
            ensure!(identity.active, Error::<T>::IdentityNotActive);
            ensure!(
                !VerificationQueue::<T>::contains_key(&who),
                Error::<T>::VerificationAlreadyPending
            );

            let now: u64 = T::TimeProvider::now().unique_saturated_into();
            VerificationQueue::<T>::insert(&who, now);

            Identities::<T>::try_mutate(&who, |maybe_identity| -> DispatchResult {
                if let Some(identity) = maybe_identity {
                    identity.verification_status = VerificationStatus::Pending;
                }
                Ok(())
            })?;

            Self::deposit_event(Event::VerificationRequested { account: who });

            Ok(())
        }

        /// Verify an identity (restricted to Auditor role or sudo)
        #[pallet::call_index(3)]
        #[pallet::weight(10_000)]
        pub fn verify_identity(
            origin: OriginFor<T>,
            target: T::AccountId,
        ) -> DispatchResult {
            let verifier = ensure_signed(origin)?;

            // Check verifier is authorized (Auditor role)
            let verifier_identity = Identities::<T>::get(&verifier).ok_or(Error::<T>::NotAuthorized)?;
            ensure!(
                matches!(verifier_identity.role, UserRole::Auditor),
                Error::<T>::NotAuthorized
            );

            Identities::<T>::try_mutate(&target, |maybe_identity| -> DispatchResult {
                let identity = maybe_identity.as_mut().ok_or(Error::<T>::IdentityNotFound)?;
                identity.verification_status = VerificationStatus::Verified;
                identity.updated_at = T::TimeProvider::now().unique_saturated_into();
                Ok(())
            })?;

            // Remove from verification queue
            VerificationQueue::<T>::remove(&target);

            Self::deposit_event(Event::IdentityVerified {
                account: target,
                verifier,
            });

            Ok(())
        }

        /// Reject identity verification
        #[pallet::call_index(4)]
        #[pallet::weight(10_000)]
        pub fn reject_verification(
            origin: OriginFor<T>,
            target: T::AccountId,
            reason: BoundedVec<u8, ConstU32<128>>,
        ) -> DispatchResult {
            let verifier = ensure_signed(origin)?;

            // Check verifier is authorized
            let verifier_identity = Identities::<T>::get(&verifier).ok_or(Error::<T>::NotAuthorized)?;
            ensure!(
                matches!(verifier_identity.role, UserRole::Auditor),
                Error::<T>::NotAuthorized
            );

            Identities::<T>::try_mutate(&target, |maybe_identity| -> DispatchResult {
                let identity = maybe_identity.as_mut().ok_or(Error::<T>::IdentityNotFound)?;
                identity.verification_status = VerificationStatus::Rejected;
                identity.updated_at = T::TimeProvider::now().unique_saturated_into();
                Ok(())
            })?;

            VerificationQueue::<T>::remove(&target);

            Self::deposit_event(Event::VerificationRejected {
                account: target,
                reason,
            });

            Ok(())
        }

        /// Deactivate identity
        #[pallet::call_index(5)]
        #[pallet::weight(10_000)]
        pub fn deactivate_identity(origin: OriginFor<T>) -> DispatchResult {
            let who = ensure_signed(origin)?;

            Identities::<T>::try_mutate(&who, |maybe_identity| -> DispatchResult {
                let identity = maybe_identity.as_mut().ok_or(Error::<T>::IdentityNotFound)?;
                identity.active = false;
                identity.updated_at = T::TimeProvider::now().unique_saturated_into();
                Ok(())
            })?;

            Self::deposit_event(Event::IdentityDeactivated { account: who });

            Ok(())
        }
    }

    // Helper functions
    impl<T: Config> Pallet<T> {
        /// Check if an account has an active identity
        pub fn is_active_identity(account: &T::AccountId) -> bool {
            if let Some(identity) = Identities::<T>::get(account) {
                identity.active
            } else {
                false
            }
        }

        /// Get identity by DID
        pub fn get_identity_by_did(did: &BoundedVec<u8, ConstU32<100>>) -> Option<Identity<T>> {
            if let Some(account) = DidToAccount::<T>::get(did) {
                Identities::<T>::get(&account)
            } else {
                None
            }
        }

        /// Check if account has specific role
        pub fn has_role(account: &T::AccountId, role: UserRole) -> bool {
            if let Some(identity) = Identities::<T>::get(account) {
                identity.role == role && identity.active
            } else {
                false
            }
        }

        /// Check if identity is verified
        pub fn is_verified(account: &T::AccountId) -> bool {
            if let Some(identity) = Identities::<T>::get(account) {
                matches!(identity.verification_status, VerificationStatus::Verified) && identity.active
            } else {
                false
            }
        }
    }
}
