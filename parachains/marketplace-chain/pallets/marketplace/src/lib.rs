//! # Marketplace Pallet
//!
//! ## Overview
//!
//! The Marketplace pallet handles purchase transactions and payment processing.
//! It provides functionality for:
//! - Purchase request and fulfillment
//! - Payment escrow and settlement
//! - Revenue distribution (provider, platform fee)
//! - Subscription management
//! - Refund processing
//!
//! ## Architecture Reference
//! See parachain.md Section: "Marketplace Chain - Marketplace"

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

    /// Purchase status
    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub enum PurchaseStatus {
        /// Purchase pending payment
        Pending,
        /// Payment completed, awaiting fulfillment
        Paid,
        /// Purchase fulfilled and access granted
        Fulfilled,
        /// Purchase cancelled
        Cancelled,
        /// Purchase refunded
        Refunded,
        /// Purchase disputed
        Disputed,
    }

    /// Purchase record
    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    #[scale_info(skip_type_params(T))]
    pub struct Purchase<T: Config> {
        /// Purchase ID
        pub purchase_id: H256,
        /// Listing ID
        pub listing_id: H256,
        /// Buyer account
        pub buyer: T::AccountId,
        /// Provider account
        pub provider: T::AccountId,
        /// Purchase amount
        pub amount: u128,
        /// Platform fee
        pub platform_fee: u128,
        /// Provider receives
        pub provider_amount: u128,
        /// Purchase status
        pub status: PurchaseStatus,
        /// Purchase timestamp
        pub purchased_at: u64,
        /// Fulfilled timestamp
        pub fulfilled_at: Option<u64>,
        /// Access expires at (for subscriptions)
        pub expires_at: Option<u64>,
    }

    /// Subscription
    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    #[scale_info(skip_type_params(T))]
    pub struct Subscription<T: Config> {
        /// Subscription ID
        pub subscription_id: H256,
        /// Listing ID
        pub listing_id: H256,
        /// Subscriber account
        pub subscriber: T::AccountId,
        /// Provider account
        pub provider: T::AccountId,
        /// Subscription amount per period
        pub amount: u128,
        /// Period in days
        pub period_days: u32,
        /// Started timestamp
        pub started_at: u64,
        /// Next payment due
        pub next_payment_at: u64,
        /// Expires at
        pub expires_at: u64,
        /// Whether active
        pub active: bool,
    }

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// The overarching event type
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// Time provider for timestamps
        type TimeProvider: Time;

        /// Platform fee percentage (e.g., 5 = 5%)
        #[pallet::constant]
        type PlatformFeePercent: Get<u8>;
    }

    /// Storage for purchases by purchase_id
    #[pallet::storage]
    #[pallet::getter(fn purchases)]
    pub type Purchases<T: Config> = StorageMap<_, Blake2_128Concat, H256, Purchase<T>>;

    /// Storage for buyer's purchases
    #[pallet::storage]
    #[pallet::getter(fn buyer_purchases)]
    pub type BuyerPurchases<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        BoundedVec<H256, ConstU32<1000>>,
        ValueQuery,
    >;

    /// Storage for provider's sales
    #[pallet::storage]
    #[pallet::getter(fn provider_sales)]
    pub type ProviderSales<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        BoundedVec<H256, ConstU32<1000>>,
        ValueQuery,
    >;

    /// Storage for subscriptions
    #[pallet::storage]
    #[pallet::getter(fn subscriptions)]
    pub type Subscriptions<T: Config> = StorageMap<_, Blake2_128Concat, H256, Subscription<T>>;

    /// Storage for active subscriptions by subscriber
    #[pallet::storage]
    #[pallet::getter(fn subscriber_subscriptions)]
    pub type SubscriberSubscriptions<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        BoundedVec<H256, ConstU32<100>>,
        ValueQuery,
    >;

    /// Purchase counter
    #[pallet::storage]
    #[pallet::getter(fn purchase_count)]
    pub type PurchaseCount<T: Config> = StorageValue<_, u64, ValueQuery>;

    /// Subscription counter
    #[pallet::storage]
    #[pallet::getter(fn subscription_count)]
    pub type SubscriptionCount<T: Config> = StorageValue<_, u64, ValueQuery>;

    /// Platform revenue collected
    #[pallet::storage]
    #[pallet::getter(fn platform_revenue)]
    pub type PlatformRevenue<T: Config> = StorageValue<_, u128, ValueQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Purchase created [purchase_id, listing_id, buyer, amount]
        PurchaseCreated {
            purchase_id: H256,
            listing_id: H256,
            buyer: T::AccountId,
            amount: u128,
        },
        /// Purchase fulfilled [purchase_id]
        PurchaseFulfilled { purchase_id: H256 },
        /// Purchase cancelled [purchase_id]
        PurchaseCancelled { purchase_id: H256 },
        /// Purchase refunded [purchase_id]
        PurchaseRefunded { purchase_id: H256 },
        /// Subscription created [subscription_id, listing_id, subscriber]
        SubscriptionCreated {
            subscription_id: H256,
            listing_id: H256,
            subscriber: T::AccountId,
        },
        /// Subscription renewed [subscription_id]
        SubscriptionRenewed { subscription_id: H256 },
        /// Subscription cancelled [subscription_id]
        SubscriptionCancelled { subscription_id: H256 },
        /// Platform fee collected [amount]
        PlatformFeeCollected { amount: u128 },
    }

    #[pallet::error]
    pub enum Error<T> {
        /// Purchase not found
        PurchaseNotFound,
        /// Subscription not found
        SubscriptionNotFound,
        /// Not authorized
        NotAuthorized,
        /// Invalid amount
        InvalidAmount,
        /// Purchase already fulfilled
        AlreadyFulfilled,
        /// Purchase not paid
        NotPaid,
        /// Insufficient balance
        InsufficientBalance,
        /// Subscription already exists
        SubscriptionAlreadyExists,
        /// Subscription not active
        SubscriptionNotActive,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Create a purchase
        #[pallet::call_index(0)]
        #[pallet::weight(10_000)]
        pub fn create_purchase(
            origin: OriginFor<T>,
            listing_id: H256,
            provider: T::AccountId,
            amount: u128,
        ) -> DispatchResult {
            let buyer = ensure_signed(origin)?;

            ensure!(amount > 0, Error::<T>::InvalidAmount);

            let now = T::TimeProvider::now().as_secs();

            // Calculate platform fee
            let platform_fee = Self::calculate_platform_fee(amount);
            let provider_amount = amount.saturating_sub(platform_fee);

            // Generate purchase ID
            let count = PurchaseCount::<T>::get();
            let purchase_id = Self::generate_purchase_id(&buyer, count);
            PurchaseCount::<T>::put(count.saturating_add(1));

            let purchase = Purchase {
                purchase_id,
                listing_id,
                buyer: buyer.clone(),
                provider: provider.clone(),
                amount,
                platform_fee,
                provider_amount,
                status: PurchaseStatus::Pending,
                purchased_at: now,
                fulfilled_at: None,
                expires_at: None,
            };

            Purchases::<T>::insert(purchase_id, purchase);

            // Add to buyer's purchases
            let mut buyer_purchases = BuyerPurchases::<T>::get(&buyer);
            let _ = buyer_purchases.try_push(purchase_id);
            BuyerPurchases::<T>::insert(&buyer, buyer_purchases);

            // Add to provider's sales
            let mut provider_sales = ProviderSales::<T>::get(&provider);
            let _ = provider_sales.try_push(purchase_id);
            ProviderSales::<T>::insert(&provider, provider_sales);

            Self::deposit_event(Event::PurchaseCreated {
                purchase_id,
                listing_id,
                buyer,
                amount,
            });

            Ok(())
        }

        /// Fulfill a purchase (grant access)
        #[pallet::call_index(1)]
        #[pallet::weight(10_000)]
        pub fn fulfill_purchase(
            origin: OriginFor<T>,
            purchase_id: H256,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            Purchases::<T>::try_mutate(purchase_id, |maybe_purchase| -> DispatchResult {
                let purchase = maybe_purchase.as_mut().ok_or(Error::<T>::PurchaseNotFound)?;

                ensure!(purchase.provider == who, Error::<T>::NotAuthorized);
                ensure!(purchase.status == PurchaseStatus::Paid, Error::<T>::NotPaid);

                let now = T::TimeProvider::now().as_secs();
                purchase.status = PurchaseStatus::Fulfilled;
                purchase.fulfilled_at = Some(now);

                // Collect platform fee
                let current_revenue = PlatformRevenue::<T>::get();
                PlatformRevenue::<T>::put(current_revenue.saturating_add(purchase.platform_fee));

                Self::deposit_event(Event::PurchaseFulfilled { purchase_id });
                Self::deposit_event(Event::PlatformFeeCollected {
                    amount: purchase.platform_fee,
                });

                Ok(())
            })
        }

        /// Mark purchase as paid (simulated payment for now)
        #[pallet::call_index(2)]
        #[pallet::weight(10_000)]
        pub fn mark_paid(origin: OriginFor<T>, purchase_id: H256) -> DispatchResult {
            let who = ensure_signed(origin)?;

            Purchases::<T>::try_mutate(purchase_id, |maybe_purchase| -> DispatchResult {
                let purchase = maybe_purchase.as_mut().ok_or(Error::<T>::PurchaseNotFound)?;

                ensure!(purchase.buyer == who, Error::<T>::NotAuthorized);
                ensure!(purchase.status == PurchaseStatus::Pending, Error::<T>::AlreadyFulfilled);

                purchase.status = PurchaseStatus::Paid;

                Ok(())
            })
        }

        /// Cancel a purchase
        #[pallet::call_index(3)]
        #[pallet::weight(10_000)]
        pub fn cancel_purchase(origin: OriginFor<T>, purchase_id: H256) -> DispatchResult {
            let who = ensure_signed(origin)?;

            Purchases::<T>::try_mutate(purchase_id, |maybe_purchase| -> DispatchResult {
                let purchase = maybe_purchase.as_mut().ok_or(Error::<T>::PurchaseNotFound)?;

                ensure!(purchase.buyer == who, Error::<T>::NotAuthorized);
                ensure!(
                    purchase.status == PurchaseStatus::Pending,
                    Error::<T>::AlreadyFulfilled
                );

                purchase.status = PurchaseStatus::Cancelled;

                Self::deposit_event(Event::PurchaseCancelled { purchase_id });

                Ok(())
            })
        }

        /// Create a subscription
        #[pallet::call_index(4)]
        #[pallet::weight(10_000)]
        pub fn create_subscription(
            origin: OriginFor<T>,
            listing_id: H256,
            provider: T::AccountId,
            amount: u128,
            period_days: u32,
        ) -> DispatchResult {
            let subscriber = ensure_signed(origin)?;

            ensure!(amount > 0, Error::<T>::InvalidAmount);
            ensure!(period_days > 0, Error::<T>::InvalidAmount);

            let now = T::TimeProvider::now().as_secs();

            // Generate subscription ID
            let count = SubscriptionCount::<T>::get();
            let subscription_id = Self::generate_subscription_id(&subscriber, count);
            SubscriptionCount::<T>::put(count.saturating_add(1));

            let next_payment_at = now + (period_days as u64 * 86400);
            let expires_at = next_payment_at;

            let subscription = Subscription {
                subscription_id,
                listing_id,
                subscriber: subscriber.clone(),
                provider,
                amount,
                period_days,
                started_at: now,
                next_payment_at,
                expires_at,
                active: true,
            };

            Subscriptions::<T>::insert(subscription_id, subscription);

            // Add to subscriber's subscriptions
            let mut subscriber_subscriptions = SubscriberSubscriptions::<T>::get(&subscriber);
            let _ = subscriber_subscriptions.try_push(subscription_id);
            SubscriberSubscriptions::<T>::insert(&subscriber, subscriber_subscriptions);

            Self::deposit_event(Event::SubscriptionCreated {
                subscription_id,
                listing_id,
                subscriber,
            });

            Ok(())
        }

        /// Cancel a subscription
        #[pallet::call_index(5)]
        #[pallet::weight(10_000)]
        pub fn cancel_subscription(
            origin: OriginFor<T>,
            subscription_id: H256,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            Subscriptions::<T>::try_mutate(subscription_id, |maybe_subscription| -> DispatchResult {
                let subscription = maybe_subscription
                    .as_mut()
                    .ok_or(Error::<T>::SubscriptionNotFound)?;

                ensure!(subscription.subscriber == who, Error::<T>::NotAuthorized);
                ensure!(subscription.active, Error::<T>::SubscriptionNotActive);

                subscription.active = false;

                Self::deposit_event(Event::SubscriptionCancelled { subscription_id });

                Ok(())
            })
        }
    }

    // Helper functions
    impl<T: Config> Pallet<T> {
        /// Generate unique purchase ID
        fn generate_purchase_id(buyer: &T::AccountId, nonce: u64) -> H256 {
            use sp_runtime::traits::Hash;
            let mut data = buyer.encode();
            data.extend_from_slice(&nonce.encode());
            data.extend_from_slice(b"purchase");
            T::Hashing::hash(&data)
        }

        /// Generate unique subscription ID
        fn generate_subscription_id(subscriber: &T::AccountId, nonce: u64) -> H256 {
            use sp_runtime::traits::Hash;
            let mut data = subscriber.encode();
            data.extend_from_slice(&nonce.encode());
            data.extend_from_slice(b"subscription");
            T::Hashing::hash(&data)
        }

        /// Calculate platform fee
        fn calculate_platform_fee(amount: u128) -> u128 {
            let fee_percent = T::PlatformFeePercent::get() as u128;
            amount.saturating_mul(fee_percent) / 100
        }

        /// Check if user has active access to listing
        pub fn has_active_access(
            listing_id: &H256,
            user: &T::AccountId,
            now: u64,
        ) -> bool {
            // Check for fulfilled purchases
            let buyer_purchases = BuyerPurchases::<T>::get(user);
            for purchase_id in buyer_purchases.iter() {
                if let Some(purchase) = Purchases::<T>::get(purchase_id) {
                    if purchase.listing_id == *listing_id
                        && purchase.status == PurchaseStatus::Fulfilled
                    {
                        if let Some(expires_at) = purchase.expires_at {
                            if now < expires_at {
                                return true;
                            }
                        } else {
                            return true; // No expiry = permanent access
                        }
                    }
                }
            }

            // Check for active subscriptions
            let subscriber_subscriptions = SubscriberSubscriptions::<T>::get(user);
            for subscription_id in subscriber_subscriptions.iter() {
                if let Some(subscription) = Subscriptions::<T>::get(subscription_id) {
                    if subscription.listing_id == *listing_id
                        && subscription.active
                        && now < subscription.expires_at
                    {
                        return true;
                    }
                }
            }

            false
        }
    }
}
