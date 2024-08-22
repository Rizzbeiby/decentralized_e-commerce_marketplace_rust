# Decentralized E-commerce Marketplace

## Overview

The Decentralized E-commerce Marketplace is a blockchain-based platform designed to enable trustless transactions between buyers and sellers without the need for intermediaries. This project leverages the capabilities of the Internet Computer to provide a secure, efficient, and transparent environment for online trading. The marketplace supports fundamental e-commerce functionalities, enhanced with blockchain-specific features such as smart contracts, escrow management, and dispute resolution.

## Features

### 1. **Product Management**

- **Create Product:** Sellers can list products on the marketplace by providing product details such as name, description, price, and stock quantity.
- **View Products:** Users can browse the list of available products.
- **Update Product:** Sellers can update the details of their listed products.
- **Delete Product:** Sellers can remove their products from the marketplace.

### 2. **Order Management**

- **Create Order:** Buyers can place orders for products. Each order is associated with a unique order ID and includes details such as the buyer's information, product ID, quantity, and total price.
- **View Orders:** Users can view the orders they have placed.
- **Update Order:** Orders can be updated by the buyer before they are processed.
- **Delete Order:** Buyers can cancel their orders if they haven't been processed yet.

### 3. **User Management**

- **Create User:** New users can register on the platform by providing a username, email, and role (buyer, seller, or admin).
- **View Users:** Admins can view the list of all registered users.
- **Update User:** Users can update their profile information.
- **Delete User:** Users can delete their accounts from the platform.

### 4. **Escrow Management**

- **Create Escrow:** An escrow is created automatically when a buyer places an order. The funds are held in escrow until the transaction is completed.
- **Release Escrow:** Once the buyer confirms the receipt of the product, the funds in escrow are released to the seller.
- **Refund Escrow:** In case of a dispute or cancellation, the funds held in escrow can be refunded to the buyer.

### 5. **Dispute Resolution**

- **Initiate Dispute:** Buyers or sellers can initiate a dispute if there is an issue with the transaction.
- **Resolve Dispute:** Disputes are resolved by an admin, who can choose to complete the order or refund the buyer.

### 6. **Product History**

- **Track Product History:** Each product's history is recorded, including its creation, updates, and transactions. This provides a transparent view of the product's lifecycle.

### 7. **Batch Tracking**

- **Batch Management:** Products can be grouped into batches, allowing for easier tracking and management of large inventories. Each batch has a unique ID and contains a set of products.

### 8. **Supplier Management**

- **Create Supplier:** Suppliers can be registered on the platform, linking them to the products they supply.
- **View Suppliers:** Admins and sellers can view the list of all suppliers.
- **Update Supplier:** Supplier information can be updated as needed.
- **Delete Supplier:** Suppliers can be removed from the platform if they are no longer active.

## Input Validation

All user inputs are validated to ensure data integrity and security. For instance, when creating a user, the system checks that the username, email, and role are valid. Similarly, when handling orders or escrow transactions, the system verifies that all required fields are correctly filled out and that the values make sense (e.g., non-zero amounts for escrow).

## Installation and Setup

To deploy this canister:

1. **Clone the Repository:**

   ```bash
   git clone <repository_url>
   cd decentralized-ecommerce-marketplace
   ```

2. **Deploy the Canister:**
   Follow the standard steps for deploying canisters on the Internet Computer. This typically involves using the DFINITY SDK to build and deploy the canister.

3. **Interact with the Canister:**
   Use the candid interface or a front-end application to interact with the deployed canister, managing users, products, orders, and other entities.

## Usage

The platform can be used by various stakeholders:

- **Buyers** can browse products, place orders, and manage their profiles.
- **Sellers** can list products, manage their inventories, and handle orders.
- **Admins** oversee the platform, resolve disputes, and manage users and suppliers.

## Contributing

Contributions to improve the Decentralized E-commerce Marketplace are welcome. Please fork the repository and submit a pull request with your enhancements.

## License

This project is licensed under the MIT License. See the LICENSE file for details.
