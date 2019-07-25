#![cfg_attr(not(feature = "std"), no_std)]

use parity_codec::{Decode, Encode};
/// A runtime module template with necessary imports
/// For more guidance on Substrate modules, see the example module
/// https://github.com/paritytech/substrate/blob/v1.0/srml/example/src/lib.rs
use support::{decl_event, decl_module, decl_storage, dispatch::Result, StorageMap, StorageValue};
use system::ensure_signed;
use support::runtime_primitives::traits::Hash;
use rstd::vec::Vec;

/// The module's configuration trait.
pub trait Trait: system::Trait {
    /// The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

#[derive(Encode, Decode, Default, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct Edit {
    ipfs_addr: IpfsAddress,
}

#[derive(Encode, Decode, Default, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct GameRules {
    num_critique_credits: u32,
    num_votes: u32,
    initial_time_limit: u64,
}

pub type IpfsAddress = Vec<u8>;

// This module's storage items.
decl_storage! {
    trait Store for Module<T: Trait> as TemplateModule {
        Edits: map T::Hash => Edit;
        PendingEdits: map (T::AccountId, u32) => T::Hash;
        NumPendingEdits: map T::AccountId => u32;
        ApprovedEdits: map (T::AccountId, u32) => T::Hash;
        NumApprovedEdits: map T::AccountId => u32;

        CurrentPic: Option<T::Hash>;
        // Number of critique credits required to
        // submit and edit.
        NumCritiqueCredits get(num_critique_credits): u32;
        // Number of votes required for a edit to be accepted
        NumVotes get(num_votes): u32;
        // Amount of time before a edit is accepted
        // regardless of votes.
        // This is number of milliseconds
        InitialTimeLimit get(initial_time_limit): u64;
        Nonce: u64;
    }
}

decl_module! {
    /// The module declaration.
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        // Initializing events
        // this is needed only if you are using events in your module
        fn deposit_event<T>() = default;

        pub fn submit_edit(origin, ipfs_addr: IpfsAddress) -> Result {
            let who = ensure_signed(origin)?;

            // Checks
            let count = <NumPendingEdits<T>>::get(&who);
            let new_count = count.checked_add(1)
                .ok_or("Overflow on number of pending edits")?;

            let edit = Edit {
                ipfs_addr,
            };

            let nonce = <Nonce<T>>::get();
            let edit_id = (<system::Module<T>>::random_seed(), &who, nonce)
                .using_encoded(<T as system::Trait>::Hashing::hash);

            // Touches storage
            <Edits<T>>::insert(edit_id, edit);
            <PendingEdits<T>>::insert(&(who.clone(), count), &edit_id);
            <NumPendingEdits<T>>::insert(&who, new_count);
            <Nonce<T>>::mutate(|n| *n += 1);

            Self::deposit_event(RawEvent::SubmitEdit(edit_id, who));
            Ok(())
        }

        pub fn approve_edit(origin, edit: T::Hash) -> Result {
            let who = ensure_signed(origin)?;
            // Checks
            let num_edits = <NumApprovedEdits<T>>::get(&who);
            let new_num_edits = num_edits.checked_add(1)
                .ok_or("Overflow on num approved edits")?;
            let num_pending_edits = <NumPendingEdits<T>>::get(&who);
            let new_pending_edits = num_pending_edits.checked_sub(1)
                .ok_or("Underflow on pending edits")?;

            // Touch Storage
            <CurrentPic<T>>::put(&edit);
            <NumApprovedEdits<T>>::insert(&who, new_num_edits);
            <NumPendingEdits<T>>::insert(who, new_pending_edits);
            Self::deposit_event(RawEvent::ApproveEdit(edit));
            Ok(())
        }

        pub fn set_initial_rules(origin, rules: GameRules) -> Result {
            let who = ensure_signed(origin)?;
            let GameRules {
                num_critique_credits,
                num_votes,
                initial_time_limit,
            } = rules;
            // Touches Storage
            <NumCritiqueCredits<T>>::put(num_critique_credits);
            <NumVotes<T>>::put(num_votes);
            <InitialTimeLimit<T>>::put(initial_time_limit);
            Ok(())
        }

        pub fn vote(origin, edit_id: T::Hash) -> Result {
            let who = ensure_signed(origin)?;
            Ok(())
        }

    }
}

decl_event!(
    pub enum Event<T>
    where
        AccountId = <T as system::Trait>::AccountId,
        <T as system::Trait>::Hash,
    {
        SubmitEdit(Hash, AccountId),
        ApproveEdit(Hash),
    }
);

/// tests for this module
#[cfg(test)]
mod tests {
    use super::*;

    use primitives::{Blake2Hasher, H256};
    use runtime_io::with_externalities;
    use runtime_primitives::{
        testing::{Digest, DigestItem, Header},
        traits::{BlakeTwo256, IdentityLookup},
        BuildStorage,
    };
    use support::{assert_ok, impl_outer_origin};

    impl_outer_origin! {
        pub enum Origin for Test {}
    }

    // For testing the module, we construct most of a mock runtime. This means
    // first constructing a configuration type (`Test`) which `impl`s each of the
    // configuration traits of modules we want to use.
    #[derive(Clone, Eq, PartialEq)]
    pub struct Test;
    impl system::Trait for Test {
        type Origin = Origin;
        type Index = u64;
        type BlockNumber = u64;
        type Hash = H256;
        type Hashing = BlakeTwo256;
        type Digest = Digest;
        type AccountId = u64;
        type Lookup = IdentityLookup<Self::AccountId>;
        type Header = Header;
        type Event = ();
        type Log = DigestItem;
    }
    impl Trait for Test {
        type Event = ();
    }
    type TemplateModule = Module<Test>;

    // This function basically just builds a genesis storage key/value store according to
    // our desired mockup.
    fn new_test_ext() -> runtime_io::TestExternalities<Blake2Hasher> {
        system::GenesisConfig::<Test>::default()
            .build_storage()
            .unwrap()
            .0
            .into()
    }

    #[test]
    fn it_works_for_default_value() {
        with_externalities(&mut new_test_ext(), || {
            let ipfs_addr = "QmWATWQ7fVPP2EFGu71UkfnqhYXDYH566qy47CnJDgvs8u".as_bytes().to_owned();
            assert_ok!(TemplateModule::submit_edit(Origin::signed(1), ipfs_addr.clone()));
            let edit = Edit{ ipfs_addr: ipfs_addr.clone() };
            let edit_id = <PendingEdits<Test>>::get((1, 0));
            assert_eq!(<Edits<Test>>::get(edit_id), edit.clone());
            assert_eq!(<NumPendingEdits<Test>>::get(1), 1);
            assert_eq!(<CurrentPic<Test>>::get(), None);
            assert_ok!(TemplateModule::approve_edit(Origin::signed(1), edit_id));
            let current_pic = <CurrentPic<Test>>::get();
            assert_eq!(Some(edit_id), current_pic);
            assert_eq!(<NumApprovedEdits<Test>>::get(1), 1);
            assert_eq!(<NumPendingEdits<Test>>::get(1), 0);
            let approved_edit = <ApprovedEdits<Test>>::get((1, 0));
            assert_eq!(edit_id, approved_edit);
            assert!(!<PendingEdits<Test>>::get((1, 0)).exists());

            let ipfs_addr = "QmWATWQ7fVPP2EFGu71UkfnqhYXDYH566qy47CnJDgvs8u".as_bytes().to_owned();
            assert_ok!(TemplateModule::submit_edit(Origin::signed(1), ipfs_addr.clone()));
            assert_eq!(<NumPendingEdits<Test>>::get(1), 2);
        });
    }


    #[test]
    fn set_initial_rules() {
        with_externalities(&mut new_test_ext(), || {
            let me = Origin::signed(1);
            let rules = GameRules {
                num_critique_credits: 5,
                num_votes: 5,
                initial_time_limit: 10000,
            };
            assert_ok!(TemplateModule::set_initial_rules(me, rules));
            assert_eq!(<NumCritiqueCredits<Test>>::get(), 5);
            assert_eq!(<NumVotes<Test>>::get(), 5);
            assert_eq!(<InitialTimeLimit<Test>>::get(), 10000);
        });
    }
}
