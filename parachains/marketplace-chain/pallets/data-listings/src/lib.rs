//! # Data Listings Pallet
//!
//! ## Overview
//!
//! The Data Listings pallet manages data listings in the marketplace.
//! It provides functionality for:
//! - Creating and managing data listings
//! - Setting pricing models (fixed price, subscription, pay-per-access)
//! - Listing lifecycle management (active, paused, expired)
//! - Data categorization and metadata
//!
//! ## Architecture Reference
//! See parachain.md Section: "Marketplace Chain - Data Listings"

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

    /// Data category type
    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub enum DataCategory {
        /// Genomic data
        Genomic,
        /// Clinical records
        Clinical,
        /// Laboratory results
        Laboratory,
        /// Medical imaging
        Imaging,
        /// Wearable device data
        Wearable,
        /// Pharmaceutical data
        Pharmaceutical,
        /// Research data
        Research,
        /// Population health data
        Population,
        /// Other category
        Other,
    }

    /// Pricing model type
    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub enum PricingModel {
        /// Fixed one-time price
        FixedPrice { amount: u128 },
        /// Subscription with period in days
        Subscription { amount: u128, period_days: u32 },
        /// Pay per access
        PayPerAccess { amount: u128 },
    }

    /// Listing status
    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub enum ListingStatus {
        /// Listing is active and available
        Active,
        /// Listing is paused by owner
        Paused,
        /// Listing has expired
        Expired,
        /// Listing is under review
        UnderReview,
        /// Listing is rejected
        Rejected,
    }

    /// Data listing
    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    #[scale_info(skip_type_params(T))]
    pub struct DataListing<T: Config> {
        /// Listing ID
        pub listing_id: H256,
        /// Data provider (owner)
        pub provider: T::AccountId,
        /// Listing title
        pub title: BoundedVec<u8, ConstU32<128>>,
        /// Listing description
        pub description: BoundedVec<u8, ConstU32<512>>,
        /// Data category
        pub category: DataCategory,
        /// Pricing model
        pub pricing: PricingModel,
        /// Number of records available
        pub record_count: u64,
        /// Data quality score (0-100)
        pub quality_score: u8,
        /// Listing status
        pub status: ListingStatus,
        /// Created timestamp
        pub created_at: u64,
        /// Updated timestamp
        pub updated_at: u64,
        /// Expiry timestamp (optional)
        pub expires_at: Option<u64>,
        /// Total purchases
        pub total_purchases: u64,
        /// Total revenue
        pub total_revenue: u128,
    }

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// The overarching event type
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// Time provider for timestamps
        type TimeProvider: Time;

        /// Maximum listings per provider
        #[pallet::constant]
        type MaxListingsPerProvider: Get<u32>;
    }

    /// Storage for data listings by listing_id
    #[pallet::storage]
    #[pallet::getter(fn listings)]
    pub type Listings<T: Config> = StorageMap<_, Blake2_128Concat, H256, DataListing<T>>;

    /// Storage for provider's listings
    #[pallet::storage]
    #[pallet::getter(fn provider_listings)]
    pub type ProviderListings<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        BoundedVec<H256, T::MaxListingsPerProvider>,
        ValueQuery,
    >;

    /// Storage for listings by category
    #[pallet::storage]
    #[pallet::getter(fn category_listings)]
    pub type CategoryListings<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        DataCategory,
        BoundedVec<H256, ConstU32<1000>>,
        ValueQuery,
    >;

    /// Listing counter for ID generation
    #[pallet::storage]
    #[pallet::getter(fn listing_count)]
    pub type ListingCount<T: Config> = StorageValue<_, u64, ValueQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Listing created [listing_id, provider]
        ListingCreated {
            listing_id: H256,
            provider: T::AccountId,
        },
        /// Listing updated [listing_id]
        ListingUpdated { listing_id: H256 },
        /// Listing status changed [listing_id, new_status]
        ListingStatusChanged {
            listing_id: H256,
            status: ListingStatus,
        },
        /// Listing purchased [listing_id, buyer, amount]
        ListingPurchased {
            listing_id: H256,
            buyer: T::AccountId,
            amount: u128,
        },
        /// Listing removed [listing_id]
        ListingRemoved { listing_id: H256 },
    }

    #[pallet::error]
    pub enum Error<T> {
        /// Listing not found
        ListingNotFound,
        /// Not authorized
        NotAuthorized,
        /// Maximum listings reached
        MaxListingsReached,
        /// Listing not active
        ListingNotActive,
        /// Invalid quality score
        InvalidQualityScore,
        /// Listing expired
        ListingExpired,
        /// Invalid pricing
        InvalidPricing,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Create a new data listing
        #[pallet::call_index(0)]
        #[pallet::weight(10_000)]
        pub fn create_listing(
            origin: OriginFor<T>,
            title: BoundedVec<u8, ConstU32<128>>,
            description: BoundedVec<u8, ConstU32<512>>,
            category: DataCategory,
            pricing: PricingModel,
            record_count: u64,
            expires_at: Option<u64>,
        ) -> DispatchResult {
            let provider = ensure_signed(origin)?;

            let now = T::TimeProvider::now().as_secs();

            // Validate pricing
            ensure!(Self::is_valid_pricing(&pricing), Error::<T>::InvalidPricing);

            // Check if provider has reached max listings
            let mut provider_listings = ProviderListings::<T>::get(&provider);
            ensure!(
                provider_listings.len() < T::MaxListingsPerProvider::get() as usize,
                Error::<T>::MaxListingsReached
            );

            // Generate listing ID
            let count = ListingCount::<T>::get();
            let listing_id = Self::generate_listing_id(&provider, count);
            ListingCount::<T>::put(count.saturating_add(1));

            let listing = DataListing {
                listing_id,
                provider: provider.clone(),
                title,
                description,
                category: category.clone(),
                pricing,
                record_count,
                quality_score: 0, // Initial score, can be updated later
                status: ListingStatus::Active,
                created_at: now,
                updated_at: now,
                expires_at,
                total_purchases: 0,
                total_revenue: 0,
            };

            Listings::<T>::insert(listing_id, listing);

            // Add to provider's listings
            provider_listings
                .try_push(listing_id)
                .map_err(|_| Error::<T>::MaxListingsReached)?;
            ProviderListings::<T>::insert(&provider, provider_listings);

            // Add to category listings
            let mut category_listings = CategoryListings::<T>::get(&category);
            let _ = category_listings.try_push(listing_id);
            CategoryListings::<T>::insert(&category, category_listings);

            Self::deposit_event(Event::ListingCreated {
                listing_id,
                provider,
            });

            Ok(())
        }

        /// Update listing details
        #[pallet::call_index(1)]
        #[pallet::weight(10_000)]
        pub fn update_listing(
            origin: OriginFor<T>,
            listing_id: H256,
            title: Option<BoundedVec<u8, ConstU32<128>>>,
            description: Option<BoundedVec<u8, ConstU32<512>>>,
            pricing: Option<PricingModel>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            Listings::<T>::try_mutate(listing_id, |maybe_listing| -> DispatchResult {
                let listing = maybe_listing.as_mut().ok_or(Error::<T>::ListingNotFound)?;

                ensure!(listing.provider == who, Error::<T>::NotAuthorized);

                let now = T::TimeProvider::now().as_secs();

                if let Some(new_title) = title {
                    listing.title = new_title;
                }

                if let Some(new_description) = description {
                    listing.description = new_description;
                }

                if let Some(new_pricing) = pricing {
                    ensure!(
                        Self::is_valid_pricing(&new_pricing),
                        Error::<T>::InvalidPricing
                    );
                    listing.pricing = new_pricing;
                }

                listing.updated_at = now;

                Self::deposit_event(Event::ListingUpdated { listing_id });

                Ok(())
            })
        }

        /// Change listing status
        #[pallet::call_index(2)]
        #[pallet::weight(10_000)]
        pub fn set_listing_status(
            origin: OriginFor<T>,
            listing_id: H256,
            status: ListingStatus,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            Listings::<T>::try_mutate(listing_id, |maybe_listing| -> DispatchResult {
                let listing = maybe_listing.as_mut().ok_or(Error::<T>::ListingNotFound)?;

                ensure!(listing.provider == who, Error::<T>::NotAuthorized);

                listing.status = status.clone();
                listing.updated_at = T::TimeProvider::now().as_secs();

                Self::deposit_event(Event::ListingStatusChanged { listing_id, status });

                Ok(())
            })
        }

        /// Update quality score (can be called by authorized oracles/reviewers)
        #[pallet::call_index(3)]
        #[pallet::weight(10_000)]
        pub fn update_quality_score(
            origin: OriginFor<T>,
            listing_id: H256,
            score: u8,
        ) -> DispatchResult {
            let _who = ensure_signed(origin)?;

            ensure!(score <= 100, Error::<T>::InvalidQualityScore);

            Listings::<T>::try_mutate(listing_id, |maybe_listing| -> DispatchResult {
                let listing = maybe_listing.as_mut().ok_or(Error::<T>::ListingNotFound)?;

                listing.quality_score = score;
                listing.updated_at = T::TimeProvider::now().as_secs();

                Self::deposit_event(Event::ListingUpdated { listing_id });

                Ok(())
            })
        }

        /// Remove a listing
        #[pallet::call_index(4)]
        #[pallet::weight(10_000)]
        pub fn remove_listing(origin: OriginFor<T>, listing_id: H256) -> DispatchResult {
            let who = ensure_signed(origin)?;

            let listing = Listings::<T>::get(listing_id).ok_or(Error::<T>::ListingNotFound)?;
            ensure!(listing.provider == who, Error::<T>::NotAuthorized);

            // Remove from provider's listings
            let mut provider_listings = ProviderListings::<T>::get(&who);
            provider_listings.retain(|id| *id != listing_id);
            ProviderListings::<T>::insert(&who, provider_listings);

            // Remove from category listings
            let mut category_listings = CategoryListings::<T>::get(&listing.category);
            category_listings.retain(|id| *id != listing_id);
            CategoryListings::<T>::insert(&listing.category, category_listings);

            // Remove listing
            Listings::<T>::remove(listing_id);

            Self::deposit_event(Event::ListingRemoved { listing_id });

            Ok(())
        }
    }

    // Helper functions
    impl<T: Config> Pallet<T> {
        /// Generate unique listing ID
        fn generate_listing_id(provider: &T::AccountId, nonce: u64) -> H256 {
            use sp_runtime::traits::Hash;
            let mut data = provider.encode();
            data.extend_from_slice(&nonce.encode());
            T::Hashing::hash(&data)
        }

        /// Validate pricing model
        fn is_valid_pricing(pricing: &PricingModel) -> bool {
            match pricing {
                PricingModel::FixedPrice { amount } => *amount > 0,
                PricingModel::Subscription {
                    amount,
                    period_days,
                } => *amount > 0 && *period_days > 0,
                PricingModel::PayPerAccess { amount } => *amount > 0,
            }
        }

        /// Record a purchase (called by marketplace pallet)
        pub fn record_purchase(listing_id: &H256, amount: u128) -> DispatchResult {
            Listings::<T>::try_mutate(listing_id, |maybe_listing| -> DispatchResult {
                let listing = maybe_listing.as_mut().ok_or(Error::<T>::ListingNotFound)?;

                listing.total_purchases = listing.total_purchases.saturating_add(1);
                listing.total_revenue = listing.total_revenue.saturating_add(amount);

                Ok(())
            })
        }

        /// Check if listing is active and available
        pub fn is_listing_available(listing_id: &H256, now: u64) -> bool {
            if let Some(listing) = Listings::<T>::get(listing_id) {
                if listing.status != ListingStatus::Active {
                    return false;
                }

                if let Some(expires_at) = listing.expires_at {
                    if now >= expires_at {
                        return false;
                    }
                }

                return true;
            }

            false
        }

        /// Get listing price
        pub fn get_listing_price(listing_id: &H256) -> Option<u128> {
            Listings::<T>::get(listing_id).map(|listing| match listing.pricing {
                PricingModel::FixedPrice { amount } => amount,
                PricingModel::Subscription { amount, .. } => amount,
                PricingModel::PayPerAccess { amount } => amount,
            })
        }
    }
}
