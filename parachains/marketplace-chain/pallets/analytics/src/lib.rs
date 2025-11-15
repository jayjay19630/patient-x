//! # Analytics Pallet
//!
//! ## Overview
//!
//! The Analytics pallet tracks marketplace usage and generates statistics.
//! It provides functionality for:
//! - Recording marketplace events and activity
//! - Tracking listing views and interactions
//! - Aggregating sales and revenue statistics
//! - Monitoring user activity and engagement
//! - Generating marketplace insights
//!
//! ## Architecture Reference
//! See parachain.md Section: "Marketplace Chain - Analytics"

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

    /// Event type for analytics
    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub enum AnalyticsEventType {
        /// Listing viewed
        ListingView,
        /// Listing clicked
        ListingClick,
        /// Purchase initiated
        PurchaseInitiated,
        /// Purchase completed
        PurchaseCompleted,
        /// Search performed
        Search,
        /// Profile viewed
        ProfileView,
    }

    /// Analytics event
    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    #[scale_info(skip_type_params(T))]
    pub struct AnalyticsEvent<T: Config> {
        /// Event ID
        pub event_id: H256,
        /// Event type
        pub event_type: AnalyticsEventType,
        /// User account (optional)
        pub user: Option<T::AccountId>,
        /// Listing ID (optional)
        pub listing_id: Option<H256>,
        /// Additional data hash (optional)
        pub data_hash: Option<H256>,
        /// Timestamp
        pub timestamp: u64,
    }

    /// Daily statistics
    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub struct DailyStats {
        /// Date (unix timestamp at start of day)
        pub date: u64,
        /// Total views
        pub total_views: u64,
        /// Total clicks
        pub total_clicks: u64,
        /// Total purchases
        pub total_purchases: u64,
        /// Total revenue
        pub total_revenue: u128,
        /// Unique visitors
        pub unique_visitors: u32,
        /// Total searches
        pub total_searches: u32,
    }

    /// Listing statistics
    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub struct ListingStats {
        /// Listing ID
        pub listing_id: H256,
        /// Total views
        pub total_views: u64,
        /// Total clicks
        pub total_clicks: u64,
        /// Total purchases
        pub total_purchases: u64,
        /// Conversion rate (purchases/views * 100)
        pub conversion_rate: u16,
        /// Last viewed
        pub last_viewed: u64,
    }

    /// User activity
    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    #[scale_info(skip_type_params(T))]
    pub struct UserActivity<T: Config> {
        /// User account
        pub user: T::AccountId,
        /// Total views
        pub total_views: u32,
        /// Total purchases
        pub total_purchases: u32,
        /// Total spent
        pub total_spent: u128,
        /// Last active
        pub last_active: u64,
        /// First seen
        pub first_seen: u64,
    }

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// The overarching event type
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// Time provider for timestamps
        type TimeProvider: Time;
    }

    /// Storage for analytics events
    #[pallet::storage]
    #[pallet::getter(fn events)]
    pub type Events<T: Config> = StorageMap<_, Blake2_128Concat, H256, AnalyticsEvent<T>>;

    /// Storage for daily statistics
    #[pallet::storage]
    #[pallet::getter(fn daily_stats)]
    pub type DailyStatistics<T: Config> = StorageMap<_, Blake2_128Concat, u64, DailyStats>;

    /// Storage for listing statistics
    #[pallet::storage]
    #[pallet::getter(fn listing_stats)]
    pub type ListingStatistics<T: Config> = StorageMap<_, Blake2_128Concat, H256, ListingStats>;

    /// Storage for user activity
    #[pallet::storage]
    #[pallet::getter(fn user_activity)]
    pub type UserActivities<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, UserActivity<T>>;

    /// Event counter
    #[pallet::storage]
    #[pallet::getter(fn event_count)]
    pub type EventCount<T: Config> = StorageValue<_, u64, ValueQuery>;

    /// Total marketplace statistics
    #[pallet::storage]
    #[pallet::getter(fn total_views)]
    pub type TotalViews<T: Config> = StorageValue<_, u64, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn total_purchases)]
    pub type TotalPurchases<T: Config> = StorageValue<_, u64, ValueQuery>;

    #[pallet::storage]
    #[pallet::getter(fn total_revenue)]
    pub type TotalRevenue<T: Config> = StorageValue<_, u128, ValueQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Event recorded [event_id, event_type]
        EventRecorded {
            event_id: H256,
            event_type: AnalyticsEventType,
        },
        /// Daily stats updated [date, total_purchases, total_revenue]
        DailyStatsUpdated {
            date: u64,
            total_purchases: u64,
            total_revenue: u128,
        },
        /// Listing stats updated [listing_id, total_views]
        ListingStatsUpdated { listing_id: H256, total_views: u64 },
    }

    #[pallet::error]
    pub enum Error<T> {
        /// Event not found
        EventNotFound,
        /// Statistics not found
        StatsNotFound,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Record an analytics event
        #[pallet::call_index(0)]
        #[pallet::weight(10_000)]
        pub fn record_event(
            origin: OriginFor<T>,
            event_type: AnalyticsEventType,
            listing_id: Option<H256>,
            data_hash: Option<H256>,
        ) -> DispatchResult {
            let user = ensure_signed(origin)?;

            let now = T::TimeProvider::now();

            // Generate event ID
            let count = EventCount::<T>::get();
            let event_id = Self::generate_event_id(&user, count);
            EventCount::<T>::put(count.saturating_add(1));

            let event = AnalyticsEvent {
                event_id,
                event_type: event_type.clone(),
                user: Some(user.clone()),
                listing_id,
                data_hash,
                timestamp: now,
            };

            Events::<T>::insert(event_id, event);

            // Update statistics based on event type
            Self::update_statistics(&event_type, listing_id.as_ref(), &user, now);

            Self::deposit_event(Event::EventRecorded {
                event_id,
                event_type,
            });

            Ok(())
        }

        /// Record a purchase for analytics (called by marketplace pallet)
        #[pallet::call_index(1)]
        #[pallet::weight(10_000)]
        pub fn record_purchase(
            origin: OriginFor<T>,
            listing_id: H256,
            buyer: T::AccountId,
            amount: u128,
        ) -> DispatchResult {
            let _who = ensure_signed(origin)?;

            let now = T::TimeProvider::now();

            // Update daily stats
            Self::update_daily_stats(now, 1, amount);

            // Update listing stats
            Self::update_listing_purchase(&listing_id);

            // Update user activity
            Self::update_user_purchase(&buyer, amount, now);

            // Update total counters
            let total_purchases = TotalPurchases::<T>::get();
            TotalPurchases::<T>::put(total_purchases.saturating_add(1));

            let total_revenue = TotalRevenue::<T>::get();
            TotalRevenue::<T>::put(total_revenue.saturating_add(amount));

            Ok(())
        }
    }

    // Helper functions
    impl<T: Config> Pallet<T> {
        /// Generate unique event ID
        fn generate_event_id(user: &T::AccountId, nonce: u64) -> H256 {
            use sp_runtime::traits::Hash;
            let mut data = user.encode();
            data.extend_from_slice(&nonce.encode());
            T::Hashing::hash(&data)
        }

        /// Get start of day timestamp
        fn get_day_start(timestamp: u64) -> u64 {
            (timestamp / 86400) * 86400
        }

        /// Update statistics based on event
        fn update_statistics(
            event_type: &AnalyticsEventType,
            listing_id: Option<&H256>,
            user: &T::AccountId,
            now: u64,
        ) {
            match event_type {
                AnalyticsEventType::ListingView => {
                    if let Some(lid) = listing_id {
                        Self::update_listing_view(lid, now);
                    }
                    Self::update_user_view(user, now);

                    let total_views = TotalViews::<T>::get();
                    TotalViews::<T>::put(total_views.saturating_add(1));
                }
                AnalyticsEventType::ListingClick => {
                    if let Some(lid) = listing_id {
                        Self::update_listing_click(lid);
                    }
                }
                AnalyticsEventType::Search => {
                    Self::update_daily_search(now);
                }
                _ => {}
            }
        }

        /// Update listing view statistics
        fn update_listing_view(listing_id: &H256, now: u64) {
            ListingStatistics::<T>::mutate(listing_id, |maybe_stats| {
                if let Some(stats) = maybe_stats {
                    stats.total_views = stats.total_views.saturating_add(1);
                    stats.last_viewed = now;

                    // Recalculate conversion rate
                    if stats.total_views > 0 {
                        stats.conversion_rate =
                            ((stats.total_purchases * 100) / stats.total_views) as u16;
                    }
                } else {
                    *maybe_stats = Some(ListingStats {
                        listing_id: *listing_id,
                        total_views: 1,
                        total_clicks: 0,
                        total_purchases: 0,
                        conversion_rate: 0,
                        last_viewed: now,
                    });
                }

                if let Some(stats) = maybe_stats {
                    Self::deposit_event(Event::ListingStatsUpdated {
                        listing_id: *listing_id,
                        total_views: stats.total_views,
                    });
                }
            });
        }

        /// Update listing click statistics
        fn update_listing_click(listing_id: &H256) {
            ListingStatistics::<T>::mutate(listing_id, |maybe_stats| {
                if let Some(stats) = maybe_stats {
                    stats.total_clicks = stats.total_clicks.saturating_add(1);
                }
            });
        }

        /// Update listing purchase statistics
        fn update_listing_purchase(listing_id: &H256) {
            ListingStatistics::<T>::mutate(listing_id, |maybe_stats| {
                if let Some(stats) = maybe_stats {
                    stats.total_purchases = stats.total_purchases.saturating_add(1);

                    // Recalculate conversion rate
                    if stats.total_views > 0 {
                        stats.conversion_rate =
                            ((stats.total_purchases * 100) / stats.total_views) as u16;
                    }
                }
            });
        }

        /// Update user view activity
        fn update_user_view(user: &T::AccountId, now: u64) {
            UserActivities::<T>::mutate(user, |maybe_activity| {
                if let Some(activity) = maybe_activity {
                    activity.total_views = activity.total_views.saturating_add(1);
                    activity.last_active = now;
                } else {
                    *maybe_activity = Some(UserActivity {
                        user: user.clone(),
                        total_views: 1,
                        total_purchases: 0,
                        total_spent: 0,
                        last_active: now,
                        first_seen: now,
                    });
                }
            });
        }

        /// Update user purchase activity
        fn update_user_purchase(user: &T::AccountId, amount: u128, now: u64) {
            UserActivities::<T>::mutate(user, |maybe_activity| {
                if let Some(activity) = maybe_activity {
                    activity.total_purchases = activity.total_purchases.saturating_add(1);
                    activity.total_spent = activity.total_spent.saturating_add(amount);
                    activity.last_active = now;
                }
            });
        }

        /// Update daily statistics
        fn update_daily_stats(now: u64, purchases: u64, revenue: u128) {
            let day_start = Self::get_day_start(now);

            DailyStatistics::<T>::mutate(day_start, |maybe_stats| {
                if let Some(stats) = maybe_stats {
                    stats.total_purchases = stats.total_purchases.saturating_add(purchases);
                    stats.total_revenue = stats.total_revenue.saturating_add(revenue);
                } else {
                    *maybe_stats = Some(DailyStats {
                        date: day_start,
                        total_views: 0,
                        total_clicks: 0,
                        total_purchases: purchases,
                        total_revenue: revenue,
                        unique_visitors: 0,
                        total_searches: 0,
                    });
                }

                if let Some(stats) = maybe_stats {
                    Self::deposit_event(Event::DailyStatsUpdated {
                        date: day_start,
                        total_purchases: stats.total_purchases,
                        total_revenue: stats.total_revenue,
                    });
                }
            });
        }

        /// Update daily search count
        fn update_daily_search(now: u64) {
            let day_start = Self::get_day_start(now);

            DailyStatistics::<T>::mutate(day_start, |maybe_stats| {
                if let Some(stats) = maybe_stats {
                    stats.total_searches = stats.total_searches.saturating_add(1);
                }
            });
        }

        /// Get listing view count
        pub fn get_listing_views(listing_id: &H256) -> u64 {
            ListingStatistics::<T>::get(listing_id)
                .map(|stats| stats.total_views)
                .unwrap_or(0)
        }

        /// Get listing conversion rate
        pub fn get_conversion_rate(listing_id: &H256) -> u16 {
            ListingStatistics::<T>::get(listing_id)
                .map(|stats| stats.conversion_rate)
                .unwrap_or(0)
        }
    }
}
