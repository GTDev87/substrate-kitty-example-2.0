use frame_support::{
    decl_module,
    decl_storage,
    decl_event,
    StorageValue,
    StorageMap,
    dispatch::DispatchResult,
    ensure,
    traits::{
        Randomness,
        Currency,
        ExistenceRequirement,
    },
};
use frame_system::ensure_signed;
use codec::{Encode, Decode};
use sp_runtime::{traits::{BlakeTwo256, Hash, Zero}, SaturatedConversion};
use sp_core::{
	H256,
};
use sp_std::cmp;


pub trait Trait: pallet_balances::Trait {
    type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
}

#[derive(Encode, Decode, Default, Clone, PartialEq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct Kitty<Hash, Balance> {
    id: Hash,
    dna: Hash,
    price: Balance,
    gen: u64,
}

decl_event! {
    pub enum Event<T>
    where
        <T as frame_system::Trait>::AccountId,
        <T as pallet_balances::Trait>::Balance
    {
        Created(AccountId, H256),
        PriceSet(AccountId, H256, Balance),
        Transferred(AccountId, AccountId, H256),
        Bought(AccountId, AccountId, H256, Balance),
    }
}

decl_storage! {
    trait Store for Module<T: Trait> as KittyStorage {
        Kitties get(fn kitty): map hasher(blake2_128_concat) H256 => Kitty<H256, T::Balance>;
        KittyOwner get(fn owner_of): map hasher(blake2_128_concat) H256 => Option<T::AccountId>;
        
        AllKittiesArray get(fn kitty_by_index): map hasher(blake2_128_concat) u64 => H256;
        AllKittiesCount get(fn all_kitties_count): u64;
        AllKittiesIndex: map hasher(blake2_128_concat) H256 => u64;

        OwnedKittiesArray get(fn kitty_of_owner_by_index): map hasher(blake2_128_concat) (T::AccountId, u64) => H256;
        OwnedKittiesCount get(fn owned_kitty_count): map hasher(blake2_128_concat) T::AccountId => u64;
        OwnedKittiesIndex: map hasher(blake2_128_concat) H256 => u64;

        Nonce: u64;
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        // Declare public functions here

        fn deposit_event() = default;

        #[weight = 100]
        fn create_kitty(origin) -> DispatchResult {
            let sender = ensure_signed(origin)?;

            let nonce = <Nonce>::get();
            let random_seed = <pallet_randomness_collective_flip::Module<T>>::random_seed();
            let random_hash = (random_seed, &sender, nonce).using_encoded(BlakeTwo256::hash);

            let new_kitty = Kitty {
                id: random_hash,
                dna: random_hash,
                price: 0_u64.saturated_into(),
                gen: 0_u64,
            };

            Self::mint(sender, random_hash, new_kitty)?;

            <Nonce>::mutate(|n| *n += 1);

            Ok(())
        }

        #[weight = 100]
        fn set_price(origin, kitty_id: H256, new_price: T::Balance) -> DispatchResult {
            let sender = ensure_signed(origin)?;

            ensure!(<Kitties<T>>::contains_key(kitty_id), "This cat does not exist");

            let owner = Self::owner_of(kitty_id).ok_or("No owner for this kitty")?;
            ensure!(owner == sender, "You do not own this cat");

            let mut kitty = Self::kitty(kitty_id);
            kitty.price = new_price;

            <Kitties<T>>::insert(kitty_id, kitty);

            Self::deposit_event(RawEvent::PriceSet(sender, kitty_id, new_price));

            Ok(())
        }

        #[weight = 100]
        fn transfer(origin, to: T::AccountId, kitty_id: H256) -> DispatchResult {
            let sender = ensure_signed(origin)?;

            let owner = Self::owner_of(kitty_id).ok_or("No owner for this kitty")?;
            ensure!(owner == sender, "You do not own this kitty");

            Self::transfer_from(sender, to, kitty_id)?;

            Ok(())
        }

        #[weight = 100]
        fn buy_kitty(origin, kitty_id: H256, max_price: T::Balance) -> DispatchResult {
            let sender = ensure_signed(origin)?;

            // ######
            ensure!(<Kitties<T>>::contains_key(kitty_id), "This cat does not exist");

            // ######
            let owner = Self::owner_of(kitty_id).ok_or("No owner for this kitty")?;
            ensure!(owner != sender, "You can't buy your own cat");

            let mut kitty = Self::kitty(kitty_id);

            // ######
            let kitty_price = kitty.price;
            ensure!(!kitty_price.is_zero(), "The cat you want to buy is not for sale");
            // ######
            ensure!(kitty_price <= max_price, "The cat you want to buy costs more than your max price");

            // ######
            <pallet_balances::Module<T> as Currency<_>>::transfer(&sender, &owner, kitty_price, ExistenceRequirement::AllowDeath)?;

            // ######
            Self::transfer_from(owner.clone(), sender.clone(), kitty_id)
                .expect("`owner` is shown to own the kitty; \
                `owner` must have greater than 0 kitties, so transfer cannot cause underflow; \
                `all_kitty_count` shares the same type as `owned_kitty_count` \
                and minting ensure there won't ever be more than `max()` kitties, \
                which means transfer cannot cause an overflow; \
                qed");
            
            // ######
            kitty.price = 0_u64.saturated_into();
            <Kitties<T>>::insert(kitty_id, kitty);

            // ######
            Self::deposit_event(RawEvent::Bought(sender, owner, kitty_id, kitty_price));

            Ok(())
        }

        #[weight = 100]
        fn breed_kitty(origin, kitty_id_1: H256, kitty_id_2: H256) -> DispatchResult {
            let sender = ensure_signed(origin)?;

            ensure!(!<KittyOwner<T>>::contains_key(kitty_id_1), "This cat 1 does not exist");
            ensure!(!<KittyOwner<T>>::contains_key(kitty_id_2), "This cat 2 does not exist");

            let nonce = <Nonce>::get();
            let random_seed = <pallet_randomness_collective_flip::Module<T>>::random_seed();
            let random_hash = (random_seed, &sender, nonce).using_encoded(BlakeTwo256::hash);

            let kitty_1 = Self::kitty(kitty_id_1);
            let kitty_2 = Self::kitty(kitty_id_2);

            // NOTE: Our gene splicing algorithm, feel free to make it your own
            let mut final_dna = kitty_1.dna;
            for (i, (dna_2_element, r)) in kitty_2.dna.as_ref().iter().zip(random_hash.as_ref().iter()).enumerate() {
                if r % 2 == 0 {
                    final_dna.as_mut()[i] = *dna_2_element;
                }
            }

            let new_kitty = Kitty {
                id: random_hash,
                dna: final_dna,
                price: 0_u64.saturated_into(),
                gen: cmp::max(kitty_1.gen, kitty_2.gen) + 1,
            };

            Self::mint(sender, random_hash, new_kitty)?;

            <Nonce>::mutate(|n| *n += 1);

            Ok(())
        }
    }
}

impl<T: Trait> Module<T> {
    fn mint(to: T::AccountId, kitty_id: H256, new_kitty: Kitty<H256, T::Balance>) -> DispatchResult {
        // #####
        ensure!(!<KittyOwner<T>>::contains_key(kitty_id), "Kitty already exists");

        let owned_kitty_count = Self::owned_kitty_count(&to);

        let new_owned_kitty_count = owned_kitty_count.checked_add(1)
            .ok_or("Overflow adding a new kitty to account balance")?;

        let all_kitties_count = Self::all_kitties_count();

        let new_all_kitties_count = all_kitties_count.checked_add(1)
            .ok_or("Overflow adding a new kitty to total supply")?;

        <Kitties<T>>::insert(kitty_id, new_kitty);
        <KittyOwner<T>>::insert(kitty_id, &to);

        <AllKittiesArray>::insert(all_kitties_count, kitty_id);
        <AllKittiesCount>::put(new_all_kitties_count);
        <AllKittiesIndex>::insert(kitty_id, all_kitties_count);

        <OwnedKittiesArray<T>>::insert((to.clone(), owned_kitty_count), kitty_id);
        <OwnedKittiesCount<T>>::insert(&to, new_owned_kitty_count);
        <OwnedKittiesIndex>::insert(kitty_id, owned_kitty_count);

        Self::deposit_event(RawEvent::Created(to, kitty_id));

        Ok(())
    }

    fn transfer_from(from: T::AccountId, to: T::AccountId, kitty_id: H256) -> DispatchResult {
        let owner = Self::owner_of(kitty_id).ok_or("No owner for this kitty")?;

        ensure!(owner == from, "'from' account does not own this kitty");

        let owned_kitty_count_from = Self::owned_kitty_count(&from);
        let owned_kitty_count_to = Self::owned_kitty_count(&to);

        let new_owned_kitty_count_to = owned_kitty_count_to.checked_add(1)
            .ok_or("Transfer causes overflow of 'to' kitty balance")?;

        let new_owned_kitty_count_from = owned_kitty_count_from.checked_sub(1)
            .ok_or("Transfer causes underflow of 'from' kitty balance")?;

        let kitty_index = <OwnedKittiesIndex>::get(kitty_id);
        if kitty_index != new_owned_kitty_count_from {
            let last_kitty_id = <OwnedKittiesArray<T>>::get((from.clone(), new_owned_kitty_count_from));
            <OwnedKittiesArray<T>>::insert((from.clone(), kitty_index), last_kitty_id);
            <OwnedKittiesIndex>::insert(last_kitty_id, kitty_index);
        }

        <KittyOwner<T>>::insert(&kitty_id, &to);
        <OwnedKittiesIndex>::insert(kitty_id, owned_kitty_count_to);

        <OwnedKittiesArray<T>>::remove((from.clone(), new_owned_kitty_count_from));
        <OwnedKittiesArray<T>>::insert((to.clone(), owned_kitty_count_to), kitty_id);

        <OwnedKittiesCount<T>>::insert(&from, new_owned_kitty_count_from);
        <OwnedKittiesCount<T>>::insert(&to, new_owned_kitty_count_to);

        Self::deposit_event(RawEvent::Transferred(from, to, kitty_id));

        Ok(())
    }
}