//! # Health Records Pallet
//!
//! ## Overview
//!
//! The Health Records pallet manages medical record anchoring and metadata for the Patient X platform.
//! It provides functionality for:
//! - Medical record anchoring with IPFS content hashes
//! - Metadata management for health records
//! - Support for multiple data formats (FHIR, DICOM, HL7)
//! - Record ownership and access tracking
//! - Audit trail for all record operations
//!
//! ## Architecture Reference
//! See parachain.md Section: "HealthData Chain - Health Records"

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

    /// Health record data format types
    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub enum DataFormat {
        /// FHIR (Fast Healthcare Interoperability Resources)
        FHIR,
        /// DICOM (Digital Imaging and Communications in Medicine)
        DICOM,
        /// HL7 (Health Level 7)
        HL7,
        /// Generic JSON format
        JSON,
        /// PDF document
        PDF,
        /// Other formats
        Other,
    }

    /// Health record category
    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    pub enum RecordCategory {
        /// Lab test results
        LabResults,
        /// Medical imaging (X-ray, MRI, CT, etc.)
        Imaging,
        /// Prescription records
        Prescription,
        /// Diagnosis and treatment notes
        Diagnosis,
        /// Genomic data
        Genomic,
        /// Vital signs and monitoring
        Vitals,
        /// Immunization records
        Immunization,
        /// Surgery records
        Surgery,
        /// Other medical records
        Other,
    }

    /// Health record metadata structure
    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    #[scale_info(skip_type_params(T))]
    pub struct HealthRecord<T: Config> {
        /// Unique record ID
        pub record_id: H256,
        /// Patient (data owner)
        pub patient: T::AccountId,
        /// IPFS content hash
        pub ipfs_hash: BoundedVec<u8, ConstU32<64>>,
        /// Record category
        pub category: RecordCategory,
        /// Data format
        pub format: DataFormat,
        /// Record title/description
        pub title: BoundedVec<u8, ConstU32<128>>,
        /// File size in bytes
        pub file_size: u64,
        /// Encryption key ID (reference to encryption pallet)
        pub encryption_key_id: Option<H256>,
        /// Upload timestamp
        pub uploaded_at: u64,
        /// Last accessed timestamp
        pub last_accessed: Option<u64>,
        /// Access count
        pub access_count: u32,
        /// Active status
        pub active: bool,
    }

    /// Access log entry for health records
    #[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, TypeInfo, MaxEncodedLen)]
    #[scale_info(skip_type_params(T))]
    pub struct AccessLog<T: Config> {
        /// Record ID
        pub record_id: H256,
        /// Accessor account
        pub accessor: T::AccountId,
        /// Access timestamp
        pub accessed_at: u64,
        /// Purpose of access
        pub purpose: BoundedVec<u8, ConstU32<64>>,
    }

    #[pallet::config]
    pub trait Config: frame_system::Config {
        /// The overarching event type
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// Time provider for timestamps
        type TimeProvider: Time;

        /// Maximum number of records per patient
        #[pallet::constant]
        type MaxRecordsPerPatient: Get<u32>;

        /// Maximum number of access logs per record
        #[pallet::constant]
        type MaxAccessLogsPerRecord: Get<u32>;
    }

    /// Storage for health records by record_id
    #[pallet::storage]
    #[pallet::getter(fn health_records)]
    pub type HealthRecords<T: Config> = StorageMap<_, Blake2_128Concat, H256, HealthRecord<T>>;

    /// Storage for record IDs by patient
    #[pallet::storage]
    #[pallet::getter(fn patient_records)]
    pub type PatientRecords<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        BoundedVec<H256, ConstU32<10000>>,
        ValueQuery,
    >;

    /// Storage for access logs by record_id
    #[pallet::storage]
    #[pallet::getter(fn access_logs)]
    pub type AccessLogs<T: Config> = StorageMap<
        _,
        Blake2_128Concat,
        H256,
        BoundedVec<AccessLog<T>, ConstU32<10000>>,
        ValueQuery,
    >;

    /// Record counter for generating unique IDs
    #[pallet::storage]
    #[pallet::getter(fn record_count)]
    pub type RecordCount<T: Config> = StorageValue<_, u64, ValueQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// Health record uploaded [record_id, patient, category]
        RecordUploaded {
            record_id: H256,
            patient: T::AccountId,
            category: RecordCategory,
        },
        /// Health record updated [record_id]
        RecordUpdated { record_id: H256 },
        /// Health record deactivated [record_id]
        RecordDeactivated { record_id: H256 },
        /// Health record accessed [record_id, accessor]
        RecordAccessed {
            record_id: H256,
            accessor: T::AccountId,
        },
    }

    #[pallet::error]
    pub enum Error<T> {
        /// Record not found
        RecordNotFound,
        /// Not authorized to access this record
        NotAuthorized,
        /// Record already deactivated
        RecordDeactivated,
        /// Maximum records per patient reached
        MaxRecordsReached,
        /// Maximum access logs reached
        MaxAccessLogsReached,
        /// Invalid IPFS hash
        InvalidIPFSHash,
        /// Invalid title
        InvalidTitle,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Upload a new health record
        ///
        /// Parameters:
        /// - `origin`: Patient uploading the record
        /// - `ipfs_hash`: IPFS content hash
        /// - `category`: Record category
        /// - `format`: Data format
        /// - `title`: Record title/description
        /// - `file_size`: File size in bytes
        /// - `encryption_key_id`: Optional encryption key reference
        #[pallet::call_index(0)]
        #[pallet::weight(10_000)]
        pub fn upload_record(
            origin: OriginFor<T>,
            ipfs_hash: BoundedVec<u8, ConstU32<64>>,
            category: RecordCategory,
            format: DataFormat,
            title: BoundedVec<u8, ConstU32<128>>,
            file_size: u64,
            encryption_key_id: Option<H256>,
        ) -> DispatchResult {
            let patient = ensure_signed(origin)?;

            // Validate inputs
            ensure!(!ipfs_hash.is_empty(), Error::<T>::InvalidIPFSHash);
            ensure!(!title.is_empty(), Error::<T>::InvalidTitle);

            let now = T::TimeProvider::now();

            // Generate unique record ID
            let count = RecordCount::<T>::get();
            let record_id = Self::generate_record_id(&patient, count);
            RecordCount::<T>::put(count.saturating_add(1));

            let record = HealthRecord {
                record_id,
                patient: patient.clone(),
                ipfs_hash,
                category: category.clone(),
                format,
                title,
                file_size,
                encryption_key_id,
                uploaded_at: now,
                last_accessed: None,
                access_count: 0,
                active: true,
            };

            // Store record
            HealthRecords::<T>::insert(record_id, record);

            // Update patient's record list
            PatientRecords::<T>::try_mutate(&patient, |records| -> DispatchResult {
                records.try_push(record_id).map_err(|_| Error::<T>::MaxRecordsReached)?;
                Ok(())
            })?;

            Self::deposit_event(Event::RecordUploaded {
                record_id,
                patient,
                category,
            });

            Ok(())
        }

        /// Update record metadata
        #[pallet::call_index(1)]
        #[pallet::weight(10_000)]
        pub fn update_record(
            origin: OriginFor<T>,
            record_id: H256,
            title: Option<BoundedVec<u8, ConstU32<128>>>,
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            HealthRecords::<T>::try_mutate(record_id, |maybe_record| -> DispatchResult {
                let record = maybe_record.as_mut().ok_or(Error::<T>::RecordNotFound)?;

                // Only patient can update
                ensure!(record.patient == who, Error::<T>::NotAuthorized);
                ensure!(record.active, Error::<T>::RecordDeactivated);

                if let Some(new_title) = title {
                    ensure!(!new_title.is_empty(), Error::<T>::InvalidTitle);
                    record.title = new_title;
                }

                Self::deposit_event(Event::RecordUpdated { record_id });

                Ok(())
            })
        }

        /// Deactivate a health record
        #[pallet::call_index(2)]
        #[pallet::weight(10_000)]
        pub fn deactivate_record(origin: OriginFor<T>, record_id: H256) -> DispatchResult {
            let who = ensure_signed(origin)?;

            HealthRecords::<T>::try_mutate(record_id, |maybe_record| -> DispatchResult {
                let record = maybe_record.as_mut().ok_or(Error::<T>::RecordNotFound)?;

                // Only patient can deactivate
                ensure!(record.patient == who, Error::<T>::NotAuthorized);

                record.active = false;

                Self::deposit_event(Event::RecordDeactivated { record_id });

                Ok(())
            })
        }

        /// Log access to a health record
        #[pallet::call_index(3)]
        #[pallet::weight(10_000)]
        pub fn log_access(
            origin: OriginFor<T>,
            record_id: H256,
            purpose: BoundedVec<u8, ConstU32<64>>,
        ) -> DispatchResult {
            let accessor = ensure_signed(origin)?;

            HealthRecords::<T>::try_mutate(record_id, |maybe_record| -> DispatchResult {
                let record = maybe_record.as_mut().ok_or(Error::<T>::RecordNotFound)?;

                ensure!(record.active, Error::<T>::RecordDeactivated);

                let now = T::TimeProvider::now();

                // Update access stats
                record.access_count = record.access_count.saturating_add(1);
                record.last_accessed = Some(now);

                // Add to access log
                let log_entry = AccessLog {
                    record_id,
                    accessor: accessor.clone(),
                    accessed_at: now,
                    purpose,
                };

                AccessLogs::<T>::try_mutate(record_id, |logs| -> DispatchResult {
                    logs.try_push(log_entry).map_err(|_| Error::<T>::MaxAccessLogsReached)?;
                    Ok(())
                })?;

                Self::deposit_event(Event::RecordAccessed { record_id, accessor });

                Ok(())
            })
        }
    }

    // Helper functions
    impl<T: Config> Pallet<T> {
        /// Generate unique record ID
        fn generate_record_id(patient: &T::AccountId, nonce: u64) -> H256 {
            use sp_runtime::traits::Hash;
            let mut data = patient.encode();
            data.extend_from_slice(&nonce.encode());
            T::Hashing::hash(&data)
        }

        /// Get all records for a patient
        pub fn get_patient_records(patient: &T::AccountId) -> Vec<HealthRecord<T>> {
            let record_ids = PatientRecords::<T>::get(patient);
            record_ids
                .iter()
                .filter_map(|id| HealthRecords::<T>::get(id))
                .collect()
        }

        /// Get active records for a patient
        pub fn get_active_patient_records(patient: &T::AccountId) -> Vec<HealthRecord<T>> {
            Self::get_patient_records(patient)
                .into_iter()
                .filter(|r| r.active)
                .collect()
        }
    }
}
