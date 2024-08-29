#[macro_use]
extern crate serde;
use regex;
use candid::{Decode, Encode};
use ic_cdk::api::time;
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{BoundedStorable, Cell, DefaultMemoryImpl, StableBTreeMap, Storable};
use std::{borrow::Cow, cell::RefCell};
use regex::Regex;

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
    quantity: u32,
    total_price: u64,
    status: String, // "pending", "completed", "canceled"
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

// Implementing Storable and BoundedStorable for Product, User, Escrow, and Order structs
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

// Thread-local storage for Products, Users, Escrows, and Orders
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
fn create_product(payload: ProductPayload) -> Result<Product, Error> {
    // Validate inputs
    validate_product_payload(&payload)?;

    // Ensure the seller exists and is a seller
    let seller = match _get_user(&payload.seller_id) {
        Some(user) => {
            if user.role == "seller" {
                user
            } else {
                return Err(Error::Unauthorized {
                    msg: format!("User with id={} is not authorized to add products", payload.seller_id),
                });
            }
        }
        None => {
            return Err(Error::NotFound {
                msg: format!("Seller with id={} not found", payload.seller_id),
            });
        }
    };

    // Generate a new product ID using thread-local storage access
    let id = PRODUCT_ID_COUNTER.with(|counter| {
        generate_id(counter)
    })?;

    // Create the product
    let product = Product {
        id,
        name: payload.name,
        description: payload.description,
        price: payload.price,
        stock_quantity: payload.stock_quantity,
        seller_id: seller.id,
        created_at: time(),
        updated_at: None,
    };
    do_insert_product(&product);
    Ok(product)
}


#[ic_cdk::update]
fn update_product(id: u64, payload: ProductPayload) -> Result<Product, Error> {
    // Validate inputs
    validate_product_payload(&payload)?;

    // Get the existing product
    let mut product = match _get_product(&id) {
        Some(prod) => prod,
        None => return Err(Error::NotFound {
            msg: format!("Product with id={} not found", id),
        }),
    };

    // Ensure that only the seller who owns the product can modify the product data
    if product.seller_id != payload.seller_id {
        return Err(Error::Unauthorized {
            msg: format!("User with id={} is not authorized to update this product", payload.seller_id),
        });
    }

    // Update the product
    product.name = payload.name;
    product.description = payload.description;
    product.price = payload.price;
    product.stock_quantity = payload.stock_quantity;
    product.updated_at = Some(time());
    do_insert_product(&product);
    Ok(product)
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
fn create_user(payload: UserPayload) -> Result<User, Error> {
    // Validate inputs
    validate_user_payload(&payload)?;

    // Generate a new user ID
    let id = USER_ID_COUNTER.with(|counter| {
        generate_id(counter)
    })?;

    // Create the user
    let user = User {
        id,
        name: payload.name,
        email: payload.email,
        role: payload.role,
        reputation: 100, // Start with a full reputation
        created_at: time(),
        updated_at: None,
    };
    do_insert_user(&user);
    Ok(user)
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
    // Validate inputs
    validate_user_payload(&payload)?;

    let mut user = match _get_user(&user_id) {
        Some(user) => user,
        None => return Err(Error::NotFound {
            msg: format!("User with id={} not found", user_id),
        }),
    };

    // Update the user
    user.name = payload.name;
    user.email = payload.email;
    user.role = payload.role;
    user.updated_at = Some(time());
    do_insert_user(&user);
    Ok(user)
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
fn create_order(payload: OrderPayload) -> Result<Order, Error> {
    // Validate order payload
    validate_order_payload(&payload)?;

    // Ensure the buyer and product exist
    match _get_user(&payload.user_id) {
        Some(user) => user,
        None => return Err(Error::NotFound {
            msg: format!("User with id={} not found", payload.user_id),
        }),
    };

    let product = match _get_product(&payload.product_id) {
        Some(product) => product,
        None => return Err(Error::NotFound {
            msg: format!("Product with id={} not found", payload.product_id),
        }),
    };

    // Check stock availability
    if payload.quantity > product.stock_quantity {
        return Err(Error::InvalidInput {
            msg: format!(
                "Requested quantity exceeds available stock. Available: {}",
                product.stock_quantity
            ),
        });
    }

    // Generate a new order ID using thread-local storage access
    let id = ORDER_ID_COUNTER.with(|counter| {
        generate_id(counter)
    })?;

    // Create the order
    let order = Order {
        id,
        product_id: payload.product_id,
        buyer_id: payload.user_id,
        quantity: payload.quantity,
        total_price: payload.total_price,
        status: "pending".to_string(),
        created_at: time(),
        updated_at: None,
    };
    do_insert_order(&order);

    // Deduct stock from product
    let mut updated_product = product.clone();
    updated_product.stock_quantity -= payload.quantity;
    updated_product.updated_at = Some(time());
    do_insert_product(&updated_product);

    Ok(order)
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
    // Validate order payload
    validate_order_payload(&payload)?;

    let mut order = match _get_order(&order_id) {
        Some(order) => order,
        None => return Err(Error::NotFound {
            msg: format!("Order with id={} not found", order_id),
        }),
    };

    // Ensure the order status allows updates
    if order.status != "pending" {
        return Err(Error::InvalidInput {
            msg: "Only pending orders can be updated.".to_string(),
        });
    }

    // Update the order
    order.product_id = payload.product_id;
    order.quantity = payload.quantity;
    order.total_price = payload.total_price;
    order.updated_at = Some(time());
    do_insert_order(&order);
    Ok(order)
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

#[ic_cdk::update]
fn complete_order(order_id: u64) -> Result<Order, Error> {
    let order_opt = ORDERS_STORAGE.with(|storage| storage.borrow().get(&order_id));
    let mut order = match order_opt {
        Some(o) => o,
        None => return Err(Error::NotFound {
            msg: format!("Order with id={} not found", order_id),
        }),
    };
    
    if order.status != "Pending" {
        return Err(Error::InvalidInput {
            msg: "Order is not in a pending state.".to_string(),
        });
    }
    
    order.status = "Completed".to_string();
    order.updated_at = Some(time());
    
    ORDERS_STORAGE.with(|storage| storage.borrow_mut().insert(order.id, order.clone()));
    Ok(order)
}

#[ic_cdk::update]
fn manage_inventory(product_id: u64, quantity: u32) -> Result<Product, Error> {
    let product_opt = PRODUCTS_STORAGE.with(|storage| storage.borrow().get(&product_id));
    let mut product = match product_opt {
        Some(p) => p,
        None => return Err(Error::NotFound {
            msg: format!("Product with id={} not found", product_id),
        }),
    };
    
    if quantity == 0 {
        return Err(Error::InvalidInput {
            msg: "Product quantity must be greater than zero.".to_string(),
        });
    }
    
    product.stock_quantity = quantity;
    product.updated_at = Some(time());
    PRODUCTS_STORAGE.with(|storage| storage.borrow_mut().insert(product.id, product.clone()));
    Ok(product)
}

#[ic_cdk::update]
fn handle_escrow(order_id: u64, amount: u64) -> Result<Escrow, Error> {
    if amount == 0 {
        return Err(Error::InvalidInput {
            msg: "Amount must be greater than zero.".to_string(),
        });
    }

    let id = ESCROW_ID_COUNTER
        .with(|counter| {
            let current_value = *counter.borrow().get();
            counter.borrow_mut().set(current_value + 1)
        })
        .expect("cannot increment escrow id counter");

    let escrow = Escrow {
        id,
        order_id,
        amount,
        status: "held".to_string(),
        created_at: time(),
        updated_at: None,
    };

    ESCROW_STORAGE.with(|storage| storage.borrow_mut().insert(escrow.id, escrow.clone()));
    Ok(escrow)
}

#[ic_cdk::update]
fn release_escrow(escrow_id: u64) -> Result<Escrow, Error> {
    let escrow_opt = ESCROW_STORAGE.with(|storage| storage.borrow().get(&escrow_id));
    let mut escrow = match escrow_opt {
        Some(e) => e,
        None => return Err(Error::NotFound {
            msg: format!("Escrow with id={} not found", escrow_id),
        }),
    };

    if escrow.status != "held" {
        return Err(Error::InvalidInput {
            msg: "Escrow is not in a held state.".to_string(),
        });
    }

    escrow.status = "released".to_string();
    escrow.updated_at = Some(time());

    ESCROW_STORAGE.with(|storage| storage.borrow_mut().insert(escrow.id, escrow.clone()));
    Ok(escrow)
}

#[ic_cdk::update]
fn refund_escrow(escrow_id: u64) -> Result<Escrow, Error> {
    let escrow_opt = ESCROW_STORAGE.with(|storage| storage.borrow().get(&escrow_id));
    let mut escrow = match escrow_opt {
        Some(e) => e,
        None => return Err(Error::NotFound {
            msg: format!("Escrow with id={} not found", escrow_id),
        }),
    };

    if escrow.status != "held" {
        return Err(Error::InvalidInput {
            msg: "Escrow is not in a held state.".to_string(),
        });
    }

    escrow.status = "refunded".to_string();
    escrow.updated_at = Some(time());

    ESCROW_STORAGE.with(|storage| storage.borrow_mut().insert(escrow.id, escrow.clone()));
    Ok(escrow)
}

#[ic_cdk::update]
fn resolve_dispute(order_id: u64, resolution: String) -> Result<Order, Error> {
    let order_opt = ORDERS_STORAGE.with(|storage| storage.borrow().get(&order_id));
    let mut order = match order_opt {
        Some(o) => o,
        None => return Err(Error::NotFound {
            msg: format!("Order with id={} not found", order_id),
        }),
    };

    if order.status != "Pending" && order.status != "In Dispute" {
        return Err(Error::InvalidInput {
            msg: "Order is not in a disputable state.".to_string(),
        });
    }

    match resolution.as_str() {
        "Complete" => order.status = "Completed".to_string(),
        "Refund" => {
            order.status = "Refunded".to_string();
            // Refund logic here, potentially interacting with escrow.
        },
        _ => return Err(Error::InvalidInput {
            msg: "Invalid resolution type.".to_string(),
        }),
    }

    order.updated_at = Some(time());
    ORDERS_STORAGE.with(|storage| storage.borrow_mut().insert(order.id, order.clone()));
    Ok(order)
}

fn generate_id(counter: &RefCell<IdCell>) -> Result<u64, Error> {
    // Borrow the `IdCell` from the `RefCell` for mutable access
    let mut counter_borrow = counter.borrow_mut();

    // Get the current value and increment it
    let current_value = *counter_borrow.get();
    let _ = counter_borrow.set(current_value + 1);

    // Return the new value
    Ok(current_value + 1)
}


fn validate_product_payload(payload: &ProductPayload) -> Result<(), Error> {
    if payload.name.is_empty() || payload.description.is_empty() || payload.price <= 0 || payload.stock_quantity <= 0 || payload.seller_id <= 0 {
        return Err(Error::InvalidInput {
            msg: "Product name, description, price, stock_quantity, and seller_id must be provided.".to_string(),
        });
    }
    Ok(())
}

fn validate_user_payload(payload: &UserPayload) -> Result<(), Error> {
    let email_regex = Regex::new(r"^[^\s@]+@[^\s@]+\.[^\s@]+$").unwrap();
    if payload.name.is_empty() || payload.email.is_empty() || !email_regex.is_match(&payload.email) || payload.role.is_empty() {
        return Err(Error::InvalidInput {
            msg: "Valid name, email, and role must be provided.".to_string(),
        });
    }
    Ok(())
}

fn validate_order_payload(payload: &OrderPayload) -> Result<(), Error> {
    if payload.user_id == 0 || payload.product_id == 0 || payload.quantity == 0 || payload.total_price == 0 {
        return Err(Error::InvalidInput {
            msg: "User ID, product ID, quantity, and total price must be provided.".to_string(),
        });
    }
    Ok(())
}

// Helper functions for inserting and retrieving entities
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
