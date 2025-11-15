//! # Encryption Pallet
//!
//! ## Overview
//!
//! The Encryption pallet manages encryption keys for health records.
//! It provides functionality for:
//! - Encryption key generation and management
//! - Key rotation for enhanced security
//! - Key access control and sharing
//! - Key revocation and lifecycle management
//!
//! ## Architecture Reference
//! See parachain.md Section: "HealthData Chain - Encryption"

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

    /// Encryption algorithm type
    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub enum EncryptionAlgorithm {
        /// ChaCha20-Poly1305 authenticated encryption
        ChaCha20Poly1305,
        /// AES-256-GCM authenticated encryption
        AES256GCM,
    }

    /// Key purpose/usage type
    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub enum KeyPurpose {
        /// Record encryption key
        RecordEncryption,
        /// Data encryption key (DEK)
        DataEncryption,
        /// Key encryption key (KEK)
        KeyEncryption,
    }

    /// Encryption key metadata
    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    #[scale_info(skip_type_params(T))]
    pub struct EncryptionKey<T: Config> {
        /// Key ID
        pub key_id: H256,
        /// Key owner
        pub owner: T::AccountId,
        /// Algorithm used
        pub algorithm: EncryptionAlgorithm,
        /// Key purpose
        pub purpose: KeyPurpose,
        /// Associated record ID (if any)
        pub record_id: Option<H256>,
        /// Creation timestamp
        pub created_at: u64,
        /// Expiration timestamp (optional)
        pub expires_at: Option<u64>,
        /// Whether key is active
        pub active: bool,
        /// Whether key is rotated
        pub rotated: bool,
        /// New key ID if rotated
        pub rotated_to: Option<H256>,
    }

    /// Key access grant
    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    #[scale_info(skip_type_params(T))]
    pub struct KeyAccess<T: Config> {
        /// Grantee account
        pub grantee: T::AccountId,
        /// Granted at timestamp
        pub granted_at: u64,
        /// Expires at timestamp (optional)
        pub expires_at: Option<u64>,
    }

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// The overarching event type
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// Time provider for timestamps
        type TimeProvider: Time;

        /// Maximum keys per account
        #[pallet::constant]
        type MaxKeysPerAccount: Get<u32>;

        /// Maximum key access grants per key
        #[pallet::constant]
        type MaxAccessGrantsPerKey: Get<u32>;
    }

    /// Storage for encryption keys by key_id
    #[pallet::storage]
    #[pallet::getter(fn encryption_keys)]
    pub type EncryptionKeys<T: Config> = StorageMap<_, Blake2_128Concat, H256, EncryptionKey<T>>;

    /// Storage for account's keys
    #[pallet::storage]
    #[pallet::getter(fn account_keys)]
    pub type AccountKeys<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        BoundedVec<H256, T::MaxKeysPerAccount>,
        ValueQuery,
    >;

    /// Storage for record encryption keys
    #[pallet::storage]
    #[pallet::getter(fn record_keys)]
    pub type RecordKeys<T: Config> = StorageMap<_, Blake2_128Concat, H256, H256>;

    /// Storage for key access grants (key_id -> grantee -> KeyAccess)
    #[pallet::storage]
    #[pallet::getter(fn key_access_grants)]
    pub type KeyAccessGrants<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        H256, // key_id
        Blake2_128Concat,
        T::AccountId, // grantee
        KeyAccess<T>,
    >;

    /// Key counter for ID generation
    #[pallet::storage]
    #[pallet::getter(fn key_count)]
    pub type KeyCount<T: Config> = StorageValue<_, u64, ValueQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Encryption key generated [key_id, owner]
        KeyGenerated {
            key_id: H256,
            owner: T::AccountId,
        },
        /// Key rotated [old_key_id, new_key_id, record_id]
        KeyRotated {
            old_key_id: H256,
            new_key_id: H256,
            record_id: H256,
        },
        /// Key revoked [key_id]
        KeyRevoked { key_id: H256 },
        /// Key access granted [key_id, grantee]
        KeyAccessGranted {
            key_id: H256,
            grantee: T::AccountId,
        },
        /// Key access revoked [key_id, grantee]
        KeyAccessRevoked {
            key_id: H256,
            grantee: T::AccountId,
        },
    }

    #[pallet::error]
    pub enum Error<T> {
        /// Key not found
        KeyNotFound,
        /// Not authorized
        NotAuthorized,
        /// Key already revoked
        KeyAlreadyRevoked,
        /// Key expired
        KeyExpired,
        /// Maximum keys per account reached
        MaxKeysReached,
        /// Maximum access grants reached
        MaxAccessGrantsReached,
        /// Record already has encryption key
        RecordAlreadyHasKey,
        /// No key for record
        NoKeyForRecord,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Generate a new encryption key
        #[pallet::call_index(0)]
        #[pallet::weight(10_000)]
        pub fn generate_key(
            origin: OriginFor<T>,
            algorithm: EncryptionAlgorithm,
            purpose: KeyPurpose,
            record_id: Option<H256>,
            expires_at: Option<u64>,
        ) -> DispatchResult {
            let owner = ensure_signed(origin)?;

            let now = T::TimeProvider::now().as_secs();

            // Check if account has reached max keys
            let mut account_keys = AccountKeys::<T>::get(&owner);
            ensure!(
                account_keys.len() < T::MaxKeysPerAccount::get() as usize,
                Error::<T>::MaxKeysReached
            );

            // If record_id is provided, ensure it doesn't already have a key
            if let Some(ref rid) = record_id {
                ensure!(
                    !RecordKeys::<T>::contains_key(rid),
                    Error::<T>::RecordAlreadyHasKey
                );
            }

            // Generate key ID
            let count = KeyCount::<T>::get();
            let key_id = Self::generate_key_id(&owner, count);
            KeyCount::<T>::put(count.saturating_add(1));

            let key = EncryptionKey {
                key_id,
                owner: owner.clone(),
                algorithm,
                purpose,
                record_id,
                created_at: now,
                expires_at,
                active: true,
                rotated: false,
                rotated_to: None,
            };

            EncryptionKeys::<T>::insert(key_id, key);

            // Add to account's key list
            account_keys
                .try_push(key_id)
                .map_err(|_| Error::<T>::MaxKeysReached)?;
            AccountKeys::<T>::insert(&owner, account_keys);

            // If record_id provided, map record to key
            if let Some(rid) = record_id {
                RecordKeys::<T>::insert(rid, key_id);
            }

            Self::deposit_event(Event::KeyGenerated { key_id, owner });

            Ok(())
        }

        /// Rotate encryption key for a record
        #[pallet::call_index(1)]
        #[pallet::weight(10_000)]
        pub fn rotate_key(
            origin: OriginFor<T>,
            record_id: H256,
            new_algorithm: EncryptionAlgorithm,
            expires_at: Option<u64>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let now = T::TimeProvider::now().as_secs();

            // Get old key for record
            let old_key_id = RecordKeys::<T>::get(&record_id).ok_or(Error::<T>::NoKeyForRecord)?;

            EncryptionKeys::<T>::try_mutate(old_key_id, |maybe_key| -> DispatchResult {
                let old_key = maybe_key.as_mut().ok_or(Error::<T>::KeyNotFound)?;

                // Only owner can rotate
                ensure!(old_key.owner == who, Error::<T>::NotAuthorized);

                // Mark old key as rotated
                old_key.rotated = true;
                old_key.active = false;

                // Generate new key
                let count = KeyCount::<T>::get();
                let new_key_id = Self::generate_key_id(&who, count);
                KeyCount::<T>::put(count.saturating_add(1));

                let new_key = EncryptionKey {
                    key_id: new_key_id,
                    owner: who.clone(),
                    algorithm: new_algorithm,
                    purpose: old_key.purpose.clone(),
                    record_id: Some(record_id),
                    created_at: now,
                    expires_at,
                    active: true,
                    rotated: false,
                    rotated_to: None,
                };

                // Link old key to new key
                old_key.rotated_to = Some(new_key_id);

                // Store new key
                EncryptionKeys::<T>::insert(new_key_id, new_key);

                // Update record mapping
                RecordKeys::<T>::insert(record_id, new_key_id);

                // Add to account's key list
                let mut account_keys = AccountKeys::<T>::get(&who);
                account_keys
                    .try_push(new_key_id)
                    .map_err(|_| Error::<T>::MaxKeysReached)?;
                AccountKeys::<T>::insert(&who, account_keys);

                Self::deposit_event(Event::KeyRotated {
                    old_key_id,
                    new_key_id,
                    record_id,
                });

                Ok(())
            })
        }

        /// Revoke an encryption key
        #[pallet::call_index(2)]
        #[pallet::weight(10_000)]
        pub fn revoke_key(origin: OriginFor<T>, key_id: H256) -> DispatchResult {
            let who = ensure_signed(origin)?;

            EncryptionKeys::<T>::try_mutate(key_id, |maybe_key| -> DispatchResult {
                let key = maybe_key.as_mut().ok_or(Error::<T>::KeyNotFound)?;

                ensure!(key.owner == who, Error::<T>::NotAuthorized);
                ensure!(key.active, Error::<T>::KeyAlreadyRevoked);

                key.active = false;

                Self::deposit_event(Event::KeyRevoked { key_id });

                Ok(())
            })
        }

        /// Grant access to a key
        #[pallet::call_index(3)]
        #[pallet::weight(10_000)]
        pub fn grant_key_access(
            origin: OriginFor<T>,
            key_id: H256,
            grantee: T::AccountId,
            expires_at: Option<u64>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let key = EncryptionKeys::<T>::get(key_id).ok_or(Error::<T>::KeyNotFound)?;
            ensure!(key.owner == who, Error::<T>::NotAuthorized);

            let now = T::TimeProvider::now().as_secs();

            let access = KeyAccess {
                grantee: grantee.clone(),
                granted_at: now,
                expires_at,
            };

            KeyAccessGrants::<T>::insert(key_id, &grantee, access);

            Self::deposit_event(Event::KeyAccessGranted { key_id, grantee });

            Ok(())
        }

        /// Revoke key access
        #[pallet::call_index(4)]
        #[pallet::weight(10_000)]
        pub fn revoke_key_access(
            origin: OriginFor<T>,
            key_id: H256,
            grantee: T::AccountId,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let key = EncryptionKeys::<T>::get(key_id).ok_or(Error::<T>::KeyNotFound)?;
            ensure!(key.owner == who, Error::<T>::NotAuthorized);

            KeyAccessGrants::<T>::remove(key_id, &grantee);

            Self::deposit_event(Event::KeyAccessRevoked { key_id, grantee });

            Ok(())
        }
    }

    // Helper functions
    impl<T: Config> Pallet<T> {
        /// Generate unique key ID
        fn generate_key_id(owner: &T::AccountId, nonce: u64) -> H256 {
            use sp_runtime::traits::Hash;
            let mut data = owner.encode();
            data.extend_from_slice(&nonce.encode());
            T::Hashing::hash(&data)
        }

        /// Check if account has access to key
        pub fn has_key_access(key_id: &H256, account: &T::AccountId, now: u64) -> bool {
            // Check if owner
            if let Some(key) = EncryptionKeys::<T>::get(key_id) {
                if &key.owner == account && key.active {
                    // Check expiration
                    if let Some(expires_at) = key.expires_at {
                        if now >= expires_at {
                            return false;
                        }
                    }
                    return true;
                }
            }

            // Check if has access grant
            if let Some(access) = KeyAccessGrants::<T>::get(key_id, account) {
                if let Some(expires_at) = access.expires_at {
                    return now < expires_at;
                }
                return true;
            }

            false
        }

        /// Get current active key for a record
        pub fn get_record_key(record_id: &H256) -> Option<H256> {
            RecordKeys::<T>::get(record_id)
        }
    }
}
