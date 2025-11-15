//! # Consent Manager Pallet
//!
//! ## Overview
//!
//! The Consent Manager pallet handles consent smart contracts for the Patient X marketplace.
//! It provides:
//! - Granular consent creation with purpose, duration, and data type specifications
//! - Consent revocation and expiry management
//! - Cross-chain consent queries (via XCM)
//! - Audit trail for all consent operations
//!
//! ## Architecture Reference
//! See parachain.md Section: "IdentityConsent Chain - Consent Management"

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
    use sp_runtime::traits::UniqueSaturatedInto;
    use pallet_identity_registry::{UserRole, Pallet as IdentityRegistry};

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    /// Purpose of data access
    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub enum DataPurpose {
        /// Medical research
        Research,
        /// Clinical trials
        ClinicalTrial,
        /// Treatment planning
        Treatment,
        /// Drug development
        DrugDevelopment,
        /// Public health analysis
        PublicHealth,
        /// AI/ML training
        MachineLearning,
        /// Other purposes
        Other,
    }

    /// Types of health data
    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub enum DataType {
        /// All medical records
        All,
        /// Lab results
        LabResults,
        /// Imaging data
        Imaging,
        /// Prescriptions
        Prescriptions,
        /// Diagnosis records
        Diagnosis,
        /// Genomic data
        Genomic,
        /// Vitals and monitoring
        Vitals,
        /// Demographics only
        Demographics,
    }

    /// Consent status
    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub enum ConsentStatus {
        /// Active consent
        Active,
        /// Revoked by patient
        Revoked,
        /// Expired
        Expired,
        /// Pending approval
        Pending,
    }

    /// Consent record structure
    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    #[scale_info(skip_type_params(T))]
    pub struct Consent<T: Config> {
        /// Unique consent ID
        pub consent_id: T::Hash,
        /// Data owner (patient)
        pub data_owner: T::AccountId,
        /// Data consumer (researcher/institution)
        pub data_consumer: T::AccountId,
        /// Purpose of data access
        pub purpose: DataPurpose,
        /// Allowed data types
        pub data_types: BoundedVec<DataType, ConstU32<10>>,
        /// Consent creation timestamp
        pub created_at: u64,
        /// Consent expiry timestamp (0 = no expiry)
        pub expires_at: u64,
        /// Current status
        pub status: ConsentStatus,
        /// Revocation timestamp (if revoked)
        pub revoked_at: Option<u64>,
        /// Access count
        pub access_count: u32,
        /// Last accessed timestamp
        pub last_accessed: Option<u64>,
        /// Additional constraints (hash of terms)
        pub terms_hash: T::Hash,
    }

    /// Consent access log entry
    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    #[scale_info(skip_type_params(T))]
    pub struct AccessLog<T: Config> {
        /// Consent ID
        pub consent_id: T::Hash,
        /// Accessor account
        pub accessor: T::AccountId,
        /// Access timestamp
        pub accessed_at: u64,
        /// Data accessed (hash)
        pub data_hash: T::Hash,
        /// Access approved
        pub approved: bool,
    }

    #[pallet::config]
    pub trait Config: frame_system::Config + pallet_identity_registry::Config {
        /// The overarching event type
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// Time provider for timestamps
        type TimeProvider: Time;

        /// Maximum number of data types per consent
        #[pallet::constant]
        type MaxDataTypes: Get<u32>;

        /// Maximum number of access logs to store per consent
        #[pallet::constant]
        type MaxAccessLogs: Get<u32>;
    }

    /// Storage for consents by consent_id
    #[pallet::storage]
    #[pallet::getter(fn consents)]
    pub type Consents<T: Config> = StorageMap<_, Blake2_128Concat, T::Hash, Consent<T>>;

    /// Storage for consent IDs by data owner
    #[pallet::storage]
    #[pallet::getter(fn owner_consents)]
    pub type OwnerConsents<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        BoundedVec<T::Hash, ConstU32<1000>>,
        ValueQuery,
    >;

    /// Storage for consent IDs by data consumer
    #[pallet::storage]
    #[pallet::getter(fn consumer_consents)]
    pub type ConsumerConsents<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        BoundedVec<T::Hash, ConstU32<1000>>,
        ValueQuery,
    >;

    /// Storage for access logs
    #[pallet::storage]
    #[pallet::getter(fn access_logs)]
    pub type AccessLogs<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::Hash,
        BoundedVec<AccessLog<T>, ConstU32<1000>>,
        ValueQuery,
    >;

    /// Consent counter for generating unique IDs
    #[pallet::storage]
    #[pallet::getter(fn consent_count)]
    pub type ConsentCount<T: Config> = StorageValue<_, u64, ValueQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Consent created [consent_id, owner, consumer, purpose]
        ConsentCreated {
            consent_id: T::Hash,
            owner: T::AccountId,
            consumer: T::AccountId,
            purpose: DataPurpose,
        },
        /// Consent updated [consent_id]
        ConsentUpdated { consent_id: T::Hash },
        /// Consent revoked [consent_id, revoker]
        ConsentRevoked {
            consent_id: T::Hash,
            revoker: T::AccountId,
        },
        /// Consent expired [consent_id]
        ConsentExpired { consent_id: T::Hash },
        /// Consent accessed [consent_id, accessor]
        ConsentAccessed {
            consent_id: T::Hash,
            accessor: T::AccountId,
        },
        /// Access denied [consent_id, accessor, reason]
        AccessDenied {
            consent_id: T::Hash,
            accessor: T::AccountId,
            reason: BoundedVec<u8, ConstU32<64>>,
        },
    }

    #[pallet::error]
    pub enum Error<T> {
        /// Consent not found
        ConsentNotFound,
        /// Not authorized
        NotAuthorized,
        /// Invalid identity (not registered or not patient role)
        InvalidIdentity,
        /// Invalid consumer (not researcher or institution)
        InvalidConsumer,
        /// Consent already revoked
        AlreadyRevoked,
        /// Consent expired
        ConsentExpired,
        /// Invalid expiry time
        InvalidExpiryTime,
        /// Maximum consent limit reached
        MaxConsentsReached,
        /// Maximum access logs reached
        MaxAccessLogsReached,
        /// Invalid data types
        InvalidDataTypes,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Create a new consent
        ///
        /// Parameters:
        /// - `origin`: Data owner (patient)
        /// - `consumer`: Data consumer (researcher/institution)
        /// - `purpose`: Purpose of data access
        /// - `data_types`: Allowed data types
        /// - `expires_at`: Expiry timestamp (0 for no expiry)
        /// - `terms_hash`: Hash of detailed terms and conditions
        #[pallet::call_index(0)]
        #[pallet::weight(10_000)]
        pub fn create_consent(
            origin: OriginFor<T>,
            consumer: T::AccountId,
            purpose: DataPurpose,
            data_types: BoundedVec<DataType, ConstU32<10>>,
            expires_at: u64,
            terms_hash: T::Hash,
        ) -> DispatchResult {
            let owner = ensure_signed(origin)?;

            // Verify owner is a patient
            ensure!(
                IdentityRegistry::<T>::has_role(&owner, UserRole::Patient),
                Error::<T>::InvalidIdentity
            );

            // Verify consumer is researcher or institution
            let is_valid_consumer = IdentityRegistry::<T>::has_role(&consumer, UserRole::Researcher)
                || IdentityRegistry::<T>::has_role(&consumer, UserRole::Institution);
            ensure!(is_valid_consumer, Error::<T>::InvalidConsumer);

            // Validate data types
            ensure!(!data_types.is_empty(), Error::<T>::InvalidDataTypes);

            let now: u64 = <T as Config>::TimeProvider::now().unique_saturated_into();

            // Validate expiry
            if expires_at > 0 {
                ensure!(expires_at > now, Error::<T>::InvalidExpiryTime);
            }

            // Generate unique consent ID
            let count = ConsentCount::<T>::get();
            let consent_id = Self::generate_consent_id(&owner, &consumer, count);
            ConsentCount::<T>::put(count.saturating_add(1));

            let consent = Consent {
                consent_id,
                data_owner: owner.clone(),
                data_consumer: consumer.clone(),
                purpose: purpose.clone(),
                data_types,
                created_at: now,
                expires_at,
                status: ConsentStatus::Active,
                revoked_at: None,
                access_count: 0,
                last_accessed: None,
                terms_hash,
            };

            // Store consent
            Consents::<T>::insert(consent_id, consent);

            // Update indices
            OwnerConsents::<T>::try_mutate(&owner, |consents| -> DispatchResult {
                consents.try_push(consent_id).map_err(|_| Error::<T>::MaxConsentsReached)?;
                Ok(())
            })?;

            ConsumerConsents::<T>::try_mutate(&consumer, |consents| -> DispatchResult {
                consents.try_push(consent_id).map_err(|_| Error::<T>::MaxConsentsReached)?;
                Ok(())
            })?;

            Self::deposit_event(Event::ConsentCreated {
                consent_id,
                owner,
                consumer,
                purpose,
            });

            Ok(())
        }

        /// Revoke an existing consent
        #[pallet::call_index(1)]
        #[pallet::weight(10_000)]
        pub fn revoke_consent(origin: OriginFor<T>, consent_id: T::Hash) -> DispatchResult {
            let revoker = ensure_signed(origin)?;

            Consents::<T>::try_mutate(consent_id, |maybe_consent| -> DispatchResult {
                let consent = maybe_consent.as_mut().ok_or(Error::<T>::ConsentNotFound)?;

                // Only data owner can revoke
                ensure!(consent.data_owner == revoker, Error::<T>::NotAuthorized);

                // Check not already revoked
                ensure!(
                    !matches!(consent.status, ConsentStatus::Revoked),
                    Error::<T>::AlreadyRevoked
                );

                let now: u64 = <T as Config>::TimeProvider::now().unique_saturated_into();
                consent.status = ConsentStatus::Revoked;
                consent.revoked_at = Some(now);

                Self::deposit_event(Event::ConsentRevoked { consent_id, revoker });

                Ok(())
            })
        }

        /// Update consent (extend expiry, modify data types)
        #[pallet::call_index(2)]
        #[pallet::weight(10_000)]
        pub fn update_consent(
            origin: OriginFor<T>,
            consent_id: T::Hash,
            new_expires_at: Option<u64>,
            new_data_types: Option<BoundedVec<DataType, ConstU32<10>>>,
        ) -> DispatchResult {
            let updater = ensure_signed(origin)?;

            Consents::<T>::try_mutate(consent_id, |maybe_consent| -> DispatchResult {
                let consent = maybe_consent.as_mut().ok_or(Error::<T>::ConsentNotFound)?;

                // Only data owner can update
                ensure!(consent.data_owner == updater, Error::<T>::NotAuthorized);

                // Check consent is active
                ensure!(
                    matches!(consent.status, ConsentStatus::Active),
                    Error::<T>::ConsentExpired
                );

                let now: u64 = <T as Config>::TimeProvider::now().unique_saturated_into();

                if let Some(expires_at) = new_expires_at {
                    if expires_at > 0 {
                        ensure!(expires_at > now, Error::<T>::InvalidExpiryTime);
                    }
                    consent.expires_at = expires_at;
                }

                if let Some(data_types) = new_data_types {
                    ensure!(!data_types.is_empty(), Error::<T>::InvalidDataTypes);
                    consent.data_types = data_types;
                }

                Self::deposit_event(Event::ConsentUpdated { consent_id });

                Ok(())
            })
        }

        /// Log data access (called by HealthData chain via XCM or directly)
        #[pallet::call_index(3)]
        #[pallet::weight(10_000)]
        pub fn log_access(
            origin: OriginFor<T>,
            consent_id: T::Hash,
            data_hash: T::Hash,
        ) -> DispatchResult {
            let accessor = ensure_signed(origin)?;

            Consents::<T>::try_mutate(consent_id, |maybe_consent| -> DispatchResult {
                let consent = maybe_consent.as_mut().ok_or(Error::<T>::ConsentNotFound)?;

                // Check consent is active and valid
                let now: u64 = <T as Config>::TimeProvider::now().unique_saturated_into();

                if !matches!(consent.status, ConsentStatus::Active) {
                    Self::deposit_event(Event::AccessDenied {
                        consent_id,
                        accessor,
                        reason: b"Consent not active".to_vec().try_into().unwrap(),
                    });
                    return Err(Error::<T>::ConsentExpired.into());
                }

                // Check expiry
                if consent.expires_at > 0 && consent.expires_at < now {
                    consent.status = ConsentStatus::Expired;
                    Self::deposit_event(Event::ConsentExpired { consent_id });
                    return Err(Error::<T>::ConsentExpired.into());
                }

                // Update access stats
                consent.access_count = consent.access_count.saturating_add(1);
                consent.last_accessed = Some(now);

                // Add to access log
                let log_entry = AccessLog {
                    consent_id,
                    accessor: accessor.clone(),
                    accessed_at: now,
                    data_hash,
                    approved: true,
                };

                AccessLogs::<T>::try_mutate(consent_id, |logs| -> DispatchResult {
                    logs.try_push(log_entry).map_err(|_| Error::<T>::MaxAccessLogsReached)?;
                    Ok(())
                })?;

                Self::deposit_event(Event::ConsentAccessed { consent_id, accessor });

                Ok(())
            })
        }

        /// Check if consent is valid (used by other chains via XCM)
        #[pallet::call_index(4)]
        #[pallet::weight(5_000)]
        pub fn check_consent(
            origin: OriginFor<T>,
            consent_id: T::Hash,
            accessor: T::AccountId,
        ) -> DispatchResult {
            ensure_signed(origin)?;

            let consent = Consents::<T>::get(consent_id).ok_or(Error::<T>::ConsentNotFound)?;

            // Check status
            if !matches!(consent.status, ConsentStatus::Active) {
                return Err(Error::<T>::ConsentExpired.into());
            }

            // Check expiry
            let now: u64 = <T as Config>::TimeProvider::now().unique_saturated_into();
            if consent.expires_at > 0 && consent.expires_at < now {
                return Err(Error::<T>::ConsentExpired.into());
            }

            // Check accessor is the designated consumer
            ensure!(consent.data_consumer == accessor, Error::<T>::NotAuthorized);

            Ok(())
        }
    }

    // Helper functions
    impl<T: Config> Pallet<T> {
        /// Generate unique consent ID
        fn generate_consent_id(owner: &T::AccountId, consumer: &T::AccountId, nonce: u64) -> T::Hash {
            use sp_runtime::traits::Hash;
            let mut data = owner.encode();
            data.extend_from_slice(&consumer.encode());
            data.extend_from_slice(&nonce.encode());
            T::Hashing::hash(&data)
        }

        /// Get all active consents for a data owner
        pub fn get_active_consents_for_owner(owner: &T::AccountId) -> Vec<Consent<T>> {
            let consent_ids = OwnerConsents::<T>::get(owner);
            consent_ids
                .iter()
                .filter_map(|id| Consents::<T>::get(id))
                .filter(|c| matches!(c.status, ConsentStatus::Active))
                .collect()
        }

        /// Get all active consents for a consumer
        pub fn get_active_consents_for_consumer(consumer: &T::AccountId) -> Vec<Consent<T>> {
            let consent_ids = ConsumerConsents::<T>::get(consumer);
            consent_ids
                .iter()
                .filter_map(|id| Consents::<T>::get(id))
                .filter(|c| matches!(c.status, ConsentStatus::Active))
                .collect()
        }

        /// Check if consent is valid (public helper)
        pub fn is_consent_valid(consent_id: &T::Hash, accessor: &T::AccountId, now: u64) -> bool {
            if let Some(consent) = Consents::<T>::get(consent_id) {
                matches!(consent.status, ConsentStatus::Active)
                    && (consent.expires_at == 0 || consent.expires_at > now)
                    && consent.data_consumer == *accessor
            } else {
                false
            }
        }
    }
}
