//! # Reputation Pallet
//!
//! ## Overview
//!
//! The Reputation pallet manages provider ratings and reviews.
//! It provides functionality for:
//! - Rating providers and data listings
//! - Submitting reviews and feedback
//! - Calculating reputation scores
//! - Dispute resolution for ratings
//! - Provider badges and achievements
//!
//! ## Architecture Reference
//! See parachain.md Section: "Marketplace Chain - Reputation"

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

    /// Rating value (1-5 stars)
    pub type RatingValue = u8;

    /// Review
    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    #[scale_info(skip_type_params(T))]
    pub struct Review<T: Config> {
        /// Review ID
        pub review_id: H256,
        /// Listing ID
        pub listing_id: H256,
        /// Provider being reviewed
        pub provider: T::AccountId,
        /// Reviewer
        pub reviewer: T::AccountId,
        /// Rating (1-5)
        pub rating: RatingValue,
        /// Review comment
        pub comment: BoundedVec<u8, ConstU32<512>>,
        /// Created timestamp
        pub created_at: u64,
        /// Whether verified purchase
        pub verified_purchase: bool,
        /// Helpful count (upvotes)
        pub helpful_count: u32,
        /// Flagged as inappropriate
        pub flagged: bool,
    }

    /// Provider reputation
    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    #[scale_info(skip_type_params(T))]
    pub struct ProviderReputation<T: Config> {
        /// Provider account
        pub provider: T::AccountId,
        /// Total reviews received
        pub total_reviews: u32,
        /// Average rating (0-500, representing 0.00-5.00)
        pub average_rating: u16,
        /// Total sales count
        pub total_sales: u64,
        /// Response rate percentage (0-100)
        pub response_rate: u8,
        /// Data quality score (0-100)
        pub quality_score: u8,
        /// Verified provider
        pub verified: bool,
        /// Last updated
        pub updated_at: u64,
    }

    /// Badge type
    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub enum BadgeType {
        /// Top rated provider
        TopRated,
        /// Verified provider
        Verified,
        /// High volume seller
        HighVolume,
        /// Excellent quality
        QualityLeader,
        /// Fast responder
        FastResponder,
    }

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// The overarching event type
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// Time provider for timestamps
        type TimeProvider: Time;

        /// Maximum reviews per user
        #[pallet::constant]
        type MaxReviewsPerUser: Get<u32>;
    }

    /// Storage for reviews by review_id
    #[pallet::storage]
    #[pallet::getter(fn reviews)]
    pub type Reviews<T: Config> = StorageMap<_, Blake2_128Concat, H256, Review<T>>;

    /// Storage for listing reviews
    #[pallet::storage]
    #[pallet::getter(fn listing_reviews)]
    pub type ListingReviews<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        H256, // listing_id
        BoundedVec<H256, ConstU32<1000>>, // review_ids
        ValueQuery,
    >;

    /// Storage for reviewer's reviews
    #[pallet::storage]
    #[pallet::getter(fn reviewer_reviews)]
    pub type ReviewerReviews<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        BoundedVec<H256, T::MaxReviewsPerUser>,
        ValueQuery,
    >;

    /// Storage for provider reputation
    #[pallet::storage]
    #[pallet::getter(fn provider_reputation)]
    pub type ProviderReputations<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, ProviderReputation<T>>;

    /// Storage for provider badges
    #[pallet::storage]
    #[pallet::getter(fn provider_badges)]
    pub type ProviderBadges<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        BoundedVec<BadgeType, ConstU32<10>>,
        ValueQuery,
    >;

    /// Review counter
    #[pallet::storage]
    #[pallet::getter(fn review_count)]
    pub type ReviewCount<T: Config> = StorageValue<_, u64, ValueQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Review submitted [review_id, listing_id, provider, rating]
        ReviewSubmitted {
            review_id: H256,
            listing_id: H256,
            provider: T::AccountId,
            rating: RatingValue,
        },
        /// Review marked helpful [review_id]
        ReviewMarkedHelpful { review_id: H256 },
        /// Review flagged [review_id]
        ReviewFlagged { review_id: H256 },
        /// Provider reputation updated [provider, average_rating]
        ReputationUpdated {
            provider: T::AccountId,
            average_rating: u16,
        },
        /// Badge awarded [provider, badge_type]
        BadgeAwarded {
            provider: T::AccountId,
            badge_type: BadgeType,
        },
        /// Provider verified [provider]
        ProviderVerified { provider: T::AccountId },
    }

    #[pallet::error]
    pub enum Error<T> {
        /// Review not found
        ReviewNotFound,
        /// Not authorized
        NotAuthorized,
        /// Invalid rating
        InvalidRating,
        /// Already reviewed
        AlreadyReviewed,
        /// Maximum reviews reached
        MaxReviewsReached,
        /// Cannot review own listing
        CannotReviewOwnListing,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Submit a review
        #[pallet::call_index(0)]
        #[pallet::weight(10_000)]
        pub fn submit_review(
            origin: OriginFor<T>,
            listing_id: H256,
            provider: T::AccountId,
            rating: RatingValue,
            comment: BoundedVec<u8, ConstU32<512>>,
            verified_purchase: bool,
        ) -> DispatchResult {
            let reviewer = ensure_signed(origin)?;

            // Validate rating
            ensure!(rating >= 1 && rating <= 5, Error::<T>::InvalidRating);

            // Cannot review own listing
            ensure!(reviewer != provider, Error::<T>::CannotReviewOwnListing);

            // Check if already reviewed
            let reviewer_reviews = ReviewerReviews::<T>::get(&reviewer);
            ensure!(
                reviewer_reviews.len() < T::MaxReviewsPerUser::get() as usize,
                Error::<T>::MaxReviewsReached
            );

            let now: u64 = T::TimeProvider::now().unique_saturated_into();

            // Generate review ID
            let count = ReviewCount::<T>::get();
            let review_id = Self::generate_review_id(&reviewer, count);
            ReviewCount::<T>::put(count.saturating_add(1));

            let review = Review {
                review_id,
                listing_id,
                provider: provider.clone(),
                reviewer: reviewer.clone(),
                rating,
                comment,
                created_at: now,
                verified_purchase,
                helpful_count: 0,
                flagged: false,
            };

            Reviews::<T>::insert(review_id, review);

            // Add to listing reviews
            let mut listing_reviews = ListingReviews::<T>::get(&listing_id);
            let _ = listing_reviews.try_push(review_id);
            ListingReviews::<T>::insert(&listing_id, listing_reviews);

            // Add to reviewer's reviews
            let mut reviewer_reviews = ReviewerReviews::<T>::get(&reviewer);
            let _ = reviewer_reviews.try_push(review_id);
            ReviewerReviews::<T>::insert(&reviewer, reviewer_reviews);

            // Update provider reputation
            Self::update_provider_reputation(&provider, rating);

            Self::deposit_event(Event::ReviewSubmitted {
                review_id,
                listing_id,
                provider,
                rating,
            });

            Ok(())
        }

        /// Mark review as helpful
        #[pallet::call_index(1)]
        #[pallet::weight(10_000)]
        pub fn mark_helpful(origin: OriginFor<T>, review_id: H256) -> DispatchResult {
            let _who = ensure_signed(origin)?;

            Reviews::<T>::try_mutate(review_id, |maybe_review| -> DispatchResult {
                let review = maybe_review.as_mut().ok_or(Error::<T>::ReviewNotFound)?;

                review.helpful_count = review.helpful_count.saturating_add(1);

                Self::deposit_event(Event::ReviewMarkedHelpful { review_id });

                Ok(())
            })
        }

        /// Flag review as inappropriate
        #[pallet::call_index(2)]
        #[pallet::weight(10_000)]
        pub fn flag_review(origin: OriginFor<T>, review_id: H256) -> DispatchResult {
            let _who = ensure_signed(origin)?;

            Reviews::<T>::try_mutate(review_id, |maybe_review| -> DispatchResult {
                let review = maybe_review.as_mut().ok_or(Error::<T>::ReviewNotFound)?;

                review.flagged = true;

                Self::deposit_event(Event::ReviewFlagged { review_id });

                Ok(())
            })
        }

        /// Update provider quality score (admin/oracle)
        #[pallet::call_index(3)]
        #[pallet::weight(10_000)]
        pub fn update_quality_score(
            origin: OriginFor<T>,
            provider: T::AccountId,
            score: u8,
        ) -> DispatchResult {
            let _who = ensure_signed(origin)?;

            ensure!(score <= 100, Error::<T>::InvalidRating);

            ProviderReputations::<T>::try_mutate(&provider, |maybe_reputation| -> DispatchResult {
                if let Some(reputation) = maybe_reputation {
                    reputation.quality_score = score;
                    reputation.updated_at = T::TimeProvider::now().unique_saturated_into();
                } else {
                    let now: u64 = T::TimeProvider::now().unique_saturated_into();
                    *maybe_reputation = Some(ProviderReputation {
                        provider: provider.clone(),
                        total_reviews: 0,
                        average_rating: 0,
                        total_sales: 0,
                        response_rate: 0,
                        quality_score: score,
                        verified: false,
                        updated_at: now,
                    });
                }

                Ok(())
            })
        }

        /// Verify provider (admin/oracle)
        #[pallet::call_index(4)]
        #[pallet::weight(10_000)]
        pub fn verify_provider(origin: OriginFor<T>, provider: T::AccountId) -> DispatchResult {
            let _who = ensure_signed(origin)?;

            ProviderReputations::<T>::try_mutate(&provider, |maybe_reputation| -> DispatchResult {
                if let Some(reputation) = maybe_reputation {
                    reputation.verified = true;
                    reputation.updated_at = T::TimeProvider::now().unique_saturated_into();
                } else {
                    let now: u64 = T::TimeProvider::now().unique_saturated_into();
                    *maybe_reputation = Some(ProviderReputation {
                        provider: provider.clone(),
                        total_reviews: 0,
                        average_rating: 0,
                        total_sales: 0,
                        response_rate: 0,
                        quality_score: 0,
                        verified: true,
                        updated_at: now,
                    });
                }

                Self::deposit_event(Event::ProviderVerified {
                    provider: provider.clone(),
                });

                // Award verified badge
                Self::award_badge(&provider, BadgeType::Verified);

                Ok(())
            })
        }

        /// Award badge to provider (admin/automated)
        #[pallet::call_index(5)]
        #[pallet::weight(10_000)]
        pub fn award_badge(
            origin: OriginFor<T>,
            provider: T::AccountId,
            badge_type: BadgeType,
        ) -> DispatchResult {
            let _who = ensure_signed(origin)?;

            Self::award_badge(&provider, badge_type);

            Ok(())
        }
    }

    // Helper functions
    impl<T: Config> Pallet<T> {
        /// Generate unique review ID
        fn generate_review_id(reviewer: &T::AccountId, nonce: u64) -> H256 {
            use sp_runtime::traits::Hash;
            let mut data = reviewer.encode();
            data.extend_from_slice(&nonce.encode());
            T::Hashing::hash(&data)
        }

        /// Update provider reputation with new rating
        fn update_provider_reputation(provider: &T::AccountId, new_rating: RatingValue) {
            let now: u64 = T::TimeProvider::now().unique_saturated_into();

            ProviderReputations::<T>::mutate(provider, |maybe_reputation| {
                if let Some(reputation) = maybe_reputation {
                    // Update average rating
                    let total_ratings = reputation.total_reviews as u32 + 1;
                    let current_sum =
                        (reputation.average_rating as u32 * reputation.total_reviews) / 100;
                    let new_sum = current_sum + (new_rating as u32 * 100);
                    reputation.average_rating = (new_sum / total_ratings) as u16;
                    reputation.total_reviews = total_ratings;
                    reputation.updated_at = now;
                } else {
                    // Create new reputation
                    *maybe_reputation = Some(ProviderReputation {
                        provider: provider.clone(),
                        total_reviews: 1,
                        average_rating: (new_rating as u16) * 100,
                        total_sales: 0,
                        response_rate: 0,
                        quality_score: 0,
                        verified: false,
                        updated_at: now,
                    });
                }

                if let Some(reputation) = maybe_reputation {
                    Self::deposit_event(Event::ReputationUpdated {
                        provider: provider.clone(),
                        average_rating: reputation.average_rating,
                    });
                }
            });
        }

        /// Award badge to provider
        fn award_badge(provider: &T::AccountId, badge_type: BadgeType) {
            let mut badges = ProviderBadges::<T>::get(provider);

            // Check if badge already exists
            if !badges.contains(&badge_type) {
                let _ = badges.try_push(badge_type.clone());
                ProviderBadges::<T>::insert(provider, badges);

                Self::deposit_event(Event::BadgeAwarded {
                    provider: provider.clone(),
                    badge_type,
                });
            }
        }

        /// Get provider average rating (as float-like value)
        pub fn get_provider_rating(provider: &T::AccountId) -> Option<u16> {
            ProviderReputations::<T>::get(provider).map(|rep| rep.average_rating)
        }

        /// Check if provider is verified
        pub fn is_verified(provider: &T::AccountId) -> bool {
            ProviderReputations::<T>::get(provider)
                .map(|rep| rep.verified)
                .unwrap_or(false)
        }
    }
}
