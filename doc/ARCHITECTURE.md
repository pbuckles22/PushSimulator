# **Architecture Blueprint for "Push" (Speed & Simulation Focus)**

To achieve maximum execution speed for Monte Carlo simulations, rapid early-stage visual debugging, and a native iOS final product, we will use a strict **Rust-centric architecture** with specialized bridges for different phases.

## **1\. The Core: Pure Rust Engine**

You will write the headless rules, state machine, and Monte Carlo AI bots entirely in Rust. Rust runs at the speed of C++, has zero garbage collection overhead, and prevents memory leaks. This single codebase will power *every other phase* of the project.

## **2\. Phase 1b: The "Debug Table" Viewer (WebAssembly)**

To visually watch bots play and verify the game logic *early*, we will build a web-based viewer. We will **not** use Flutter or build a heavy desktop app.

* **The Bridge:** We will use wasm-bindgen to compile the Rust engine into WebAssembly (Wasm).  
* **The Frontend:** A simple, lightweight HTML/JavaScript page.  
* **The Open-Source Assets:** We will use selfthinker/CSS-Playing-Cards (available on GitHub). This repository uses pure CSS and Unicode to render highly realistic, scalable playing cards without needing dozens of .png image files.  
* **The Result:** You open index.html in Chrome, and you can watch two AI bots play "Push" on a green CSS felt table. You can see their hands, watch the board build, and read a scrolling text log of exactly *why* they chose to push, steal, or discard.

## **3\. Phase 2: The Simulation Number Cruncher**

Once the game looks correct in the Web Viewer, you scale it up.

* You will use a Rust library called Rayon.  
* Rayon spreads the headless game loop across every CPU core of the machine that runs `push_sim`.  
* **Where it runs:** coding stays on the Windows PC. The first CPU host is the Mac: install Rust there once, pull this repo, and build `push_sim` on the Mac. Do not cross-compile a Mac binary from Windows. A Linux EC2 instance is the later scale host, the same binary and the same Rayon loop.  
* **How many games:** 1,000 to prove a batch finishes, 10,000 for a first look, and **100,000 games per matchup** when ranking bots. One game is a full 5-round match. Go to a million only when two bots are still inside half a percentage point. Detail: [requirements/MONTE_CARLO_STRATEGY.md](requirements/MONTE_CARLO_STRATEGY.md).

## **4\. Phase 3 & 4: iOS App & Multiplayer Server**

When you are ready to build the consumer iOS app, you do **not** rewrite the game.

* **The Local App Bridge:** You will use **Mozilla's UniFFI** to compile your Rust engine into a static .xcframework. You will build the iOS UI natively using **SwiftUI**, talking directly to the embedded Rust engine.  
* **The Multiplayer Server:** You will deploy a Rust web framework (Axum) to your EC2 instance. The server imports the exact same Rust core engine to handle thousands of simultaneous online iOS players over WebSockets, using a fraction of the RAM required by Python or Node.js.