# VoidCraft-Core

VoidCraft is an experimental game engine, drawing design inspiration from Bevy.

As the core component, this project primarily includes the following modules:

- [`vc_utils`] : Platform-agnostic container extensions
    - Examples include hash functions and related containers, block linked lists, circular arrays, and stack-optimized dynamic arrays.

- [`vc_os`] : Cross-platform abstraction layer for host-side programs
    - Provides specific `std` implementations for `no_std` and `wasm` environments.
    - Examples include synchronization primitives like `Mutex` and `RwLock`, offering fallback implementations based on atomic variables and spin operations.
    - Multi-threading and file system abstractions are not yet implemented. The current workaround is to restrict `no_std` environments to single-threaded operation.

- [`vc_reflect`] : Dynamic reflection system
    - Implemented via compile-time AST modification, generating code through simple metadata annotations. Main features include:
    - 1. Runtime Type Information (RTTI), such as type names and IDs, generic information, and custom attributes (similar to attributes in C#).
    - 2. Type erasure, allowing different types to be stored as identical dynamic objects, supporting dynamic content manipulation.
    - 3. Global registry containing type information for all reflected objects, supporting interface (method) reflection and automatic type registration.
    - 4. Serialization and deserialization, enabling data serialization/deserialization based on reflection information without implementing `serde`.
    - 5. Path access, allowing multi-level field access via strings.

- [`vc_task`] : Task system (multi-threading and concurrency system)
    - Provides a task pool (thread pool) for managing thread resources.
    - Offers multiple async executors that work with the task pool to automatically handle asynchronous tasks and achieve multi-threaded load balancing.
    - Provides a parallel task interface that splits continuous tasks into chunks for parallel execution via the task pool.

- [`vc_ecs`] : Entity-Component-System
    - ECS for game engines, a data-oriented programming design pattern.
    - In progress.
