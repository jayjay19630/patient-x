//! # Authentication Pallet
//!
//! ## Overview
//!
//! The Authentication pallet provides session management and API key authentication
//! for the Patient X platform.
//!
//! ## Architecture Reference
//! See parachain.md Section: "IdentityConsent Chain - Authentication"

#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::{pallet_prelude::*, traits::Time};
    use frame_system::pallet_prelude::*;
    use sp_std::prelude::*;
    use sp_runtime::traits::UniqueSaturatedInto;
    use pallet_identity_registry::Pallet as IdentityRegistry;

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    /// Authentication session
    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    #[scale_info(skip_type_params(T))]
    pub struct Session<T: Config> {
        pub account: T::AccountId,
        pub session_id: T::Hash,
        pub created_at: u64,
        pub expires_at: u64,
        pub active: bool,
    }

    /// API key for programmatic access
    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    #[scale_info(skip_type_params(T))]
    pub struct ApiKey<T: Config> {
        pub account: T::AccountId,
        pub key_hash: T::Hash,
        pub name: BoundedVec<u8, ConstU32<32>>,
        pub created_at: u64,
        pub last_used: Option<u64>,
        pub active: bool,
    }

    #[pallet::config]
    pub trait Config: frame_system::Config + pallet_identity_registry::Config {
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// Time provider for timestamps
        type TimeProvider: Time;

        #[pallet::constant]
        type SessionDuration: Get<u64>;
    }

    #[pallet::storage]
    pub type Sessions<T: Config> = StorageMap<_, Blake2_128Concat, T::Hash, Session<T>>;

    #[pallet::storage]
    pub type ApiKeys<T: Config> = StorageMap<_, Blake2_128Concat, T::Hash, ApiKey<T>>;

    #[pallet::storage]
    pub type AccountSessions<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        BoundedVec<T::Hash, ConstU32<10>>,
        ValueQuery,
    >;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        SessionCreated { account: T::AccountId, session_id: T::Hash },
        SessionRevoked { session_id: T::Hash },
        ApiKeyCreated { account: T::AccountId, key_hash: T::Hash },
        ApiKeyRevoked { key_hash: T::Hash },
    }

    #[pallet::error]
    pub enum Error<T> {
        SessionNotFound,
        SessionExpired,
        NotAuthorized,
        InvalidIdentity,
        MaxSessionsReached,
        ApiKeyNotFound,
        ApiKeyInactive,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::call_index(0)]
        #[pallet::weight(10_000)]
        pub fn create_session(origin: OriginFor<T>) -> DispatchResult {
            let who = ensure_signed(origin)?;

            ensure!(
                IdentityRegistry::<T>::is_active_identity(&who),
                Error::<T>::InvalidIdentity
            );

            let now: u64 = <T as Config>::TimeProvider::now().unique_saturated_into();
            let session_id = Self::generate_session_id(&who, now);

            let session = Session {
                account: who.clone(),
                session_id,
                created_at: now,
                expires_at: now + T::SessionDuration::get(),
                active: true,
            };

            Sessions::<T>::insert(session_id, session);

            AccountSessions::<T>::try_mutate(&who, |sessions| -> DispatchResult {
                sessions.try_push(session_id).map_err(|_| Error::<T>::MaxSessionsReached)?;
                Ok(())
            })?;

            Self::deposit_event(Event::SessionCreated { account: who, session_id });

            Ok(())
        }

        #[pallet::call_index(1)]
        #[pallet::weight(10_000)]
        pub fn revoke_session(origin: OriginFor<T>, session_id: T::Hash) -> DispatchResult {
            let who = ensure_signed(origin)?;

            Sessions::<T>::try_mutate(session_id, |maybe_session| -> DispatchResult {
                let session = maybe_session.as_mut().ok_or(Error::<T>::SessionNotFound)?;
                ensure!(session.account == who, Error::<T>::NotAuthorized);
                session.active = false;
                Ok(())
            })?;

            Self::deposit_event(Event::SessionRevoked { session_id });

            Ok(())
        }

        #[pallet::call_index(2)]
        #[pallet::weight(10_000)]
        pub fn create_api_key(
            origin: OriginFor<T>,
            key_hash: T::Hash,
            name: BoundedVec<u8, ConstU32<32>>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            ensure!(
                IdentityRegistry::<T>::is_active_identity(&who),
                Error::<T>::InvalidIdentity
            );

            let now: u64 = <T as Config>::TimeProvider::now().unique_saturated_into();

            let api_key = ApiKey {
                account: who.clone(),
                key_hash,
                name,
                created_at: now,
                last_used: None,
                active: true,
            };

            ApiKeys::<T>::insert(key_hash, api_key);

            Self::deposit_event(Event::ApiKeyCreated { account: who, key_hash });

            Ok(())
        }

        #[pallet::call_index(3)]
        #[pallet::weight(10_000)]
        pub fn revoke_api_key(origin: OriginFor<T>, key_hash: T::Hash) -> DispatchResult {
            let who = ensure_signed(origin)?;

            ApiKeys::<T>::try_mutate(key_hash, |maybe_key| -> DispatchResult {
                let key = maybe_key.as_mut().ok_or(Error::<T>::ApiKeyNotFound)?;
                ensure!(key.account == who, Error::<T>::NotAuthorized);
                key.active = false;
                Ok(())
            })?;

            Self::deposit_event(Event::ApiKeyRevoked { key_hash });

            Ok(())
        }
    }

    impl<T: Config> Pallet<T> {
        fn generate_session_id(account: &T::AccountId, timestamp: u64) -> T::Hash {
            use sp_runtime::traits::Hash;
            let mut data = account.encode();
            data.extend_from_slice(&timestamp.encode());
            T::Hashing::hash(&data)
        }

        pub fn is_session_valid(session_id: &T::Hash, now: u64) -> bool {
            if let Some(session) = Sessions::<T>::get(session_id) {
                session.active && session.expires_at > now
            } else {
                false
            }
        }
    }
}
