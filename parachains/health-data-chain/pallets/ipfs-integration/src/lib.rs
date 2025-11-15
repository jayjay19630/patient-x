//! # IPFS Integration Pallet
//!
//! ## Overview
//!
//! The IPFS Integration pallet provides IPFS content addressing and pinning for the Patient X platform.
//! It provides functionality for:
//! - IPFS content hash management
//! - Content pinning and unpinning
//! - IPFS node configuration
//! - Content availability tracking
//!
//! ## Architecture Reference
//! See parachain.md Section: "HealthData Chain - IPFS Integration"

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

    /// IPFS content status
    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub enum ContentStatus {
        /// Content is pinned and available
        Pinned,
        /// Content is unpinned
        Unpinned,
        /// Content pinning is pending
        PendingPin,
        /// Content is being unpinned
        PendingUnpin,
        /// Content verification failed
        Failed,
    }

    /// IPFS content metadata
    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    #[scale_info(skip_type_params(T))]
    pub struct IPFSContent<T: Config> {
        /// IPFS content hash (CID)
        pub ipfs_hash: BoundedVec<u8, ConstU32<64>>,
        /// Content owner
        pub owner: T::AccountId,
        /// Content size in bytes
        pub size: u64,
        /// Pin status
        pub status: ContentStatus,
        /// Pinned at timestamp
        pub pinned_at: Option<u64>,
        /// Unpinned at timestamp
        pub unpinned_at: Option<u64>,
        /// Number of times content has been pinned
        pub pin_count: u32,
    }

    /// IPFS node configuration
    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub struct IPFSNode {
        /// Node multiaddress
        pub multiaddr: BoundedVec<u8, ConstU32<256>>,
        /// Node peer ID
        pub peer_id: BoundedVec<u8, ConstU32<64>>,
        /// Is node active
        pub active: bool,
    }

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// The overarching event type
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// Time provider for timestamps
        type TimeProvider: Time;

        /// Maximum number of IPFS nodes
        #[pallet::constant]
        type MaxNodes: Get<u32>;
    }

    /// Storage for IPFS content by hash
    #[pallet::storage]
    #[pallet::getter(fn ipfs_content)]
    pub type IPFSContents<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        BoundedVec<u8, ConstU32<64>>,
        IPFSContent<T>,
    >;

    /// Storage for content hashes by owner
    #[pallet::storage]
    #[pallet::getter(fn owner_content)]
    pub type OwnerContent<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        BoundedVec<BoundedVec<u8, ConstU32<64>>, ConstU32<10000>>,
        ValueQuery,
    >;

    /// Storage for registered IPFS nodes
    #[pallet::storage]
    #[pallet::getter(fn ipfs_nodes)]
    pub type IPFSNodes<T: Config> = StorageValue<_, BoundedVec<IPFSNode, ConstU32<100>>, ValueQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Content pinned [ipfs_hash, owner]
        ContentPinned {
            ipfs_hash: BoundedVec<u8, ConstU32<64>>,
            owner: T::AccountId,
        },
        /// Content unpinned [ipfs_hash, owner]
        ContentUnpinned {
            ipfs_hash: BoundedVec<u8, ConstU32<64>>,
            owner: T::AccountId,
        },
        /// Content pin failed [ipfs_hash, owner]
        ContentPinFailed {
            ipfs_hash: BoundedVec<u8, ConstU32<64>>,
            owner: T::AccountId,
        },
        /// IPFS node added [peer_id]
        NodeAdded {
            peer_id: BoundedVec<u8, ConstU32<64>>,
        },
        /// IPFS node removed [peer_id]
        NodeRemoved {
            peer_id: BoundedVec<u8, ConstU32<64>>,
        },
    }

    #[pallet::error]
    pub enum Error<T> {
        /// Content not found
        ContentNotFound,
        /// Not authorized
        NotAuthorized,
        /// Invalid IPFS hash
        InvalidIPFSHash,
        /// Content already pinned
        AlreadyPinned,
        /// Content already unpinned
        AlreadyUnpinned,
        /// Maximum nodes reached
        MaxNodesReached,
        /// Node not found
        NodeNotFound,
        /// Invalid multiaddress
        InvalidMultiaddr,
        /// Invalid peer ID
        InvalidPeerId,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Pin IPFS content
        ///
        /// Parameters:
        /// - `origin`: Content owner
        /// - `ipfs_hash`: IPFS content hash (CID)
        /// - `size`: Content size in bytes
        #[pallet::call_index(0)]
        #[pallet::weight(10_000)]
        pub fn pin_content(
            origin: OriginFor<T>,
            ipfs_hash: BoundedVec<u8, ConstU32<64>>,
            size: u64,
        ) -> DispatchResult {
            let owner = ensure_signed(origin)?;

            ensure!(!ipfs_hash.is_empty(), Error::<T>::InvalidIPFSHash);

            let now = T::TimeProvider::now().as_secs();

            // Check if content already exists
            if let Some(mut content) = IPFSContents::<T>::get(&ipfs_hash) {
                // Content exists, update status
                ensure!(
                    content.status != ContentStatus::Pinned,
                    Error::<T>::AlreadyPinned
                );

                content.status = ContentStatus::Pinned;
                content.pinned_at = Some(now);
                content.pin_count = content.pin_count.saturating_add(1);

                IPFSContents::<T>::insert(&ipfs_hash, content);
            } else {
                // New content
                let content = IPFSContent {
                    ipfs_hash: ipfs_hash.clone(),
                    owner: owner.clone(),
                    size,
                    status: ContentStatus::Pinned,
                    pinned_at: Some(now),
                    unpinned_at: None,
                    pin_count: 1,
                };

                IPFSContents::<T>::insert(&ipfs_hash, content);

                // Add to owner's content list
                OwnerContent::<T>::try_mutate(&owner, |content_list| -> DispatchResult {
                    content_list
                        .try_push(ipfs_hash.clone())
                        .map_err(|_| Error::<T>::MaxNodesReached)?;
                    Ok(())
                })?;
            }

            Self::deposit_event(Event::ContentPinned { ipfs_hash, owner });

            Ok(())
        }

        /// Unpin IPFS content
        #[pallet::call_index(1)]
        #[pallet::weight(10_000)]
        pub fn unpin_content(
            origin: OriginFor<T>,
            ipfs_hash: BoundedVec<u8, ConstU32<64>>,
        ) -> DispatchResult {
            let owner = ensure_signed(origin)?;

            IPFSContents::<T>::try_mutate(&ipfs_hash, |maybe_content| -> DispatchResult {
                let content = maybe_content.as_mut().ok_or(Error::<T>::ContentNotFound)?;

                // Check ownership
                ensure!(content.owner == owner, Error::<T>::NotAuthorized);

                ensure!(
                    content.status != ContentStatus::Unpinned,
                    Error::<T>::AlreadyUnpinned
                );

                let now = T::TimeProvider::now().as_secs();
                content.status = ContentStatus::Unpinned;
                content.unpinned_at = Some(now);

                Self::deposit_event(Event::ContentUnpinned {
                    ipfs_hash: ipfs_hash.clone(),
                    owner: owner.clone(),
                });

                Ok(())
            })
        }

        /// Add IPFS node to network
        #[pallet::call_index(2)]
        #[pallet::weight(10_000)]
        pub fn add_node(
            origin: OriginFor<T>,
            multiaddr: BoundedVec<u8, ConstU32<256>>,
            peer_id: BoundedVec<u8, ConstU32<64>>,
        ) -> DispatchResult {
            ensure_root(origin)?;

            ensure!(!multiaddr.is_empty(), Error::<T>::InvalidMultiaddr);
            ensure!(!peer_id.is_empty(), Error::<T>::InvalidPeerId);

            let node = IPFSNode {
                multiaddr,
                peer_id: peer_id.clone(),
                active: true,
            };

            IPFSNodes::<T>::try_mutate(|nodes| -> DispatchResult {
                nodes.try_push(node).map_err(|_| Error::<T>::MaxNodesReached)?;
                Ok(())
            })?;

            Self::deposit_event(Event::NodeAdded { peer_id });

            Ok(())
        }

        /// Remove IPFS node from network
        #[pallet::call_index(3)]
        #[pallet::weight(10_000)]
        pub fn remove_node(
            origin: OriginFor<T>,
            peer_id: BoundedVec<u8, ConstU32<64>>,
        ) -> DispatchResult {
            ensure_root(origin)?;

            IPFSNodes::<T>::try_mutate(|nodes| -> DispatchResult {
                if let Some(pos) = nodes.iter().position(|n| n.peer_id == peer_id) {
                    nodes.remove(pos);
                    Self::deposit_event(Event::NodeRemoved { peer_id });
                    Ok(())
                } else {
                    Err(Error::<T>::NodeNotFound.into())
                }
            })
        }
    }

    // Helper functions
    impl<T: Config> Pallet<T> {
        /// Get all content for an owner
        pub fn get_owner_content(owner: &T::AccountId) -> Vec<IPFSContent<T>> {
            let content_hashes = OwnerContent::<T>::get(owner);
            content_hashes
                .iter()
                .filter_map(|hash| IPFSContents::<T>::get(hash))
                .collect()
        }

        /// Get pinned content for an owner
        pub fn get_pinned_content(owner: &T::AccountId) -> Vec<IPFSContent<T>> {
            Self::get_owner_content(owner)
                .into_iter()
                .filter(|c| c.status == ContentStatus::Pinned)
                .collect()
        }

        /// Check if content is pinned
        pub fn is_pinned(ipfs_hash: &BoundedVec<u8, ConstU32<64>>) -> bool {
            if let Some(content) = IPFSContents::<T>::get(ipfs_hash) {
                content.status == ContentStatus::Pinned
            } else {
                false
            }
        }
    }
}
