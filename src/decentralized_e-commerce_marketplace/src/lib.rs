#[macro_use]
extern crate serde;
use candid::{Decode, Encode};
use ic_cdk::api::time;
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{BoundedStorable, Cell, DefaultMemoryImpl, StableBTreeMap, Storable};
use std::{borrow::Cow, cell::RefCell};

type Memory = VirtualMemory<DefaultMemoryImpl>;
type IdCell = Cell<u64, Memory>;

// Represents a product listed by a seller
#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
struct Product {
    id: u64,
    name: String,
    description: String,
    price: u64,
    stock_quantity: u32,
    seller_id: u64,
    created_at: u64,
    updated_at: Option<u64>,
}

// Represents a user in the marketplace (buyer or seller)
#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
struct User {
    id: u64,
    name: String,
    email: String,
    role: String, // "buyer", "seller", or "admin"
    reputation: u8, // Reputation score out of 100
    created_at: u64,
    updated_at: Option<u64>,
}

// Represents an order placed by a buyer
#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
struct Order {
    id: u64,
    product_id: u64,
    buyer_id: u64,
    quantity: u64,
    total_price: u64,
    status: String,
    created_at: u64,
    updated_at: Option<u64>,
}

// Represents funds held in escrow during a transaction
#[derive(candid::CandidType, Clone, Serialize, Deserialize, Default)]
struct Escrow {
    id: u64,
    order_id: u64,
    amount: u64,
    status: String, // "held", "released", or "refunded"
    created_at: u64,
    updated_at: Option<u64>,
}

// Implementing Storable and BoundedStorable for Product, User, Escrow and Order structs
impl Storable for Product {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

impl BoundedStorable for Product {
    const MAX_SIZE: u32 = 1024;
    const IS_FIXED_SIZE: bool = false;
}

impl Storable for User {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

impl BoundedStorable for User {
    const MAX_SIZE: u32 = 1024;
    const IS_FIXED_SIZE: bool = false;
}

impl Storable for Order {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

impl BoundedStorable for Order {
    const MAX_SIZE: u32 = 1024;
    const IS_FIXED_SIZE: bool = false;
}

impl Storable for Escrow {
    fn to_bytes(&self) -> Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

impl BoundedStorable for Escrow {
    const MAX_SIZE: u32 = 1024;
    const IS_FIXED_SIZE: bool = false;
}

// Thread-local storage for Products, Users, Escrows and Orders
thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> = RefCell::new(
        MemoryManager::init(DefaultMemoryImpl::default())
    );

    static PRODUCT_ID_COUNTER: RefCell<IdCell> = RefCell::new(
        IdCell::init(MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(0))), 0)
            .expect("Cannot create a product ID counter")
    );

    static PRODUCTS_STORAGE: RefCell<StableBTreeMap<u64, Product, Memory>> =
        RefCell::new(StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(1)))
    ));

    static ORDER_ID_COUNTER: RefCell<IdCell> = RefCell::new(
        IdCell::init(MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(2))), 0)
            .expect("Cannot create an order ID counter")
    );

    static ORDERS_STORAGE: RefCell<StableBTreeMap<u64, Order, Memory>> =
        RefCell::new(StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(3)))
    ));

    static USER_ID_COUNTER: RefCell<IdCell> = RefCell::new(
        IdCell::init(MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(4))), 0)
            .expect("Cannot create a user ID counter")
    );

    static USERS_STORAGE: RefCell<StableBTreeMap<u64, User, Memory>> =
        RefCell::new(StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(5)))
    ));

    static ESCROW_ID_COUNTER: RefCell<IdCell> = RefCell::new(
        IdCell::init(MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(6))), 0)
            .expect("Cannot create an escrow ID counter")
    );

    static ESCROW_STORAGE: RefCell<StableBTreeMap<u64, Escrow, Memory>> =
        RefCell::new(StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(7)))
    ));
}

// Structs for payloads
#[derive(candid::CandidType, Serialize, Deserialize, Default)]
struct ProductPayload {
    name: String,
    description: String,
    price: u64,
    stock_quantity: u32,
    seller_id: u64,
}

#[derive(candid::CandidType, Serialize, Deserialize, Default)]
struct UserPayload {
    name: String,
    email: String,
    role: String,
}

#[derive(candid::CandidType, Serialize, Deserialize, Default)]
struct OrderPayload {
    user_id: u64,
    product_id: u64,
    quantity: u32,
    total_price: u64,
}

// CRUD operations for Products
#[ic_cdk::update]
fn create_product(payload: ProductPayload) -> Option<Product> {
    // Ensure all necessessary fields are filled
    if payload.name.is_empty() || payload.description.is_empty() || payload.price <= 0 || payload.stock_quantity <= 0 || payload.seller_id <= 0 {
        return Err(Error::InvalidInput {
            msg: "Product name, description, price, stock_quantity and seller_id must be provided.".to_string(),
        });
    }
    
    // Check if the seller exists and their role is 'seller'
    let seller = _get_user(&payload.seller_id);
    match seller {
        Some(user) => {
            if user.role == "seller" {
                let id = PRODUCT_ID_COUNTER
                    .with(|counter| {
                        let current_value = *counter.borrow().get();
                        counter.borrow_mut().set(current_value + 1)
                    })
                    .expect("cannot increment product id counter");
                
                let product = Product {
                    id,
                    name: payload.name,
                    description: payload.description,
                    price: payload.price,
                    stock_quantity: payload.stock_quantity,
                    created_at: time(),
                    updated_at: None,
                };
                do_insert_product(&product);
                Ok(product)
            } else {
                Err(Error::Unauthorized {
                    msg: format!("User with id={} is not authorized to add products", payload.seller_id),
                })
            }
        }
        None => Err(Error::NotFound {
            msg: format!("Seller with id={} not found", payload.seller_id),
        }),
    }
}

#[ic_cdk::update]
fn update_product(id: u64, product: ProductPayload) -> Result<Product, Error> {
    // Ensure all necessessary fields are filled
    if id <= 0 || name.is_empty() || description.is_empty() || price <= 0 || stock_quantity <= 0 || seller_id <= 0 {
        return Err(Error::InvalidInput {
            msg: "Product id, name, description, price, stock_quantity and seller_id must be provided.".to_string(),
        });
    }
    
    let product = _get_product(&id);
    match product {
        Some(prod) => {
            // Ensure that only the seller who owns the product can modify the product data
            if prod.seller_id == seller_id {
                match PRODUCTS_STORAGE.with(|products| products.borrow().get(id)) {
        Some(mut product) => {
            product.name = payload.name;
            product.description = payload.description;
            product.price = payload.price;
            product.stock_quantity = payload.stock_quantity;

            do_insert_product(&product);
            Ok(product);
        }
        None => Err(Error::NotFound {
            msg: format!("Product with id={} not found", id),
        }),
    }
            }
        }
    }
}

#[ic_cdk::query]
fn view_product(product_id: u64) -> Result<Product, Error> {
    match _get_product(&product_id) {
        Some(product) => Ok(product),
        None => Err(Error::NotFound {
            msg: format!("Product with id={} not found", product_id),
        }),
    }
}

#[ic_cdk::update]
fn delete_product(product_id: u64) -> Result<Product, Error> {
    match PRODUCTS_STORAGE.with(|products| products.borrow_mut().remove(&product_id)) {
        Some(product) => Ok(product),
        None => Err(Error::NotFound {
            msg: format!("Product with id={} not found", product_id),
        }),
    }
}

// CRUD operations for Users
#[ic_cdk::update]
fn create_user(payload: UserPayload) -> Option<User> {
    let id = generate_id();
    let user = User {
        id,
        name: payload.name,
        email: payload.email,
        role: payload.role,
        created_at: time(),
        updated_at: None,
    };
    do_insert_user(&user);
    Some(user)
}

#[ic_cdk::query]
fn view_user(user_id: u64) -> Result<User, Error> {
    match _get_user(&user_id) {
        Some(user) => Ok(user),
        None => Err(Error::NotFound {
            msg: format!("User with id={} not found", user_id),
        }),
    }
}

#[ic_cdk::update]
fn update_user(user_id: u64, payload: UserPayload) -> Result<User, Error> {
    match USERS_STORAGE.with(|users| users.borrow().get(&user_id)) {
        Some(mut user) => {
            user.name = payload.name;
            user.email = payload.email;
            user.role = payload.role;
            user.updated_at = Some(time());
            do_insert_user(&user);
            Ok(user)
        }
        None => Err(Error::NotFound {
            msg: format!("User with id={} not found", user_id),
        }),
    }
}

#[ic_cdk::update]
fn delete_user(user_id: u64) -> Result<User, Error> {
    match USERS_STORAGE.with(|users| users.borrow_mut().remove(&user_id)) {
        Some(user) => Ok(user),
        None => Err(Error::NotFound {
            msg: format!("User with id={} not found", user_id),
        }),
    }
}

// CRUD operations for Orders
#[ic_cdk::update]
fn create_order(payload: OrderPayload) -> Option<Order> {
    let id = generate_id();
    let order = Order {
        id,
        user_id: payload.user_id,
        product_id: payload.product_id,
        quantity: payload.quantity,
        total_price: payload.total_price,
        created_at: time(),
        updated_at: None,
    };
    do_insert_order(&order);
    Some(order)
}

#[ic_cdk::query]
fn view_order(order_id: u64) -> Result<Order, Error> {
    match _get_order(&order_id) {
        Some(order) => Ok(order),
        None => Err(Error::NotFound {
            msg: format!("Order with id={} not found", order_id),
        }),
    }
}

#[ic_cdk::update]
fn update_order(order_id: u64, payload: OrderPayload) -> Result<Order, Error> {
    match ORDERS_STORAGE.with(|orders| orders.borrow().get(&order_id)) {
        Some(mut order) => {
            order.product_id = payload.product_id;
            order.quantity = payload.quantity;
            order.total_price = payload.total_price;
            order.updated_at = Some(time());
            do_insert_order(&order);
            Ok(order)
        }
        None => Err(Error::NotFound {
            msg: format!("Order with id={} not found", order_id),
        }),
    }
}

#[ic_cdk::update]
fn delete_order(order_id: u64) -> Result<Order, Error> {
    match ORDERS_STORAGE.with(|orders| orders.borrow_mut().remove(&order_id)) {
        Some(order) => Ok(order),
        None => Err(Error::NotFound {
            msg: format!("Order with id={} not found", order_id),
        }),
    }
}

fn do_insert_product(product: &Product) {
    PRODUCTS_STORAGE.with(|products| products.borrow_mut().insert(product.id, product.clone()));
}

fn do_insert_user(user: &User) {
    USERS_STORAGE.with(|users| users.borrow_mut().insert(user.id, user.clone()));
}

fn do_insert_order(order: &Order) {
    ORDERS_STORAGE.with(|orders| orders.borrow_mut().insert(order.id, order.clone()));
}

fn _get_product(product_id: &u64) -> Option<Product> {
    PRODUCTS_STORAGE.with(|products| products.borrow().get(product_id))
}

fn _get_user(user_id: &u64) -> Option<User> {
    USERS_STORAGE.with(|users| users.borrow().get(user_id))
}

fn _get_order(order_id: &u64) -> Option<Order> {
    ORDERS_STORAGE.with(|orders| orders.borrow().get(order_id))
}

// Error enum for error handling
#[derive(candid::CandidType, Deserialize, Serialize)]
enum Error {
    Unauthorized { msg: String },
    NotFound { msg: String },
    InvalidInput { msg: String },
}

// Export candid for the canister
ic_cdk::export_candid!();
