# 🤖 Protopolis

**A Rust-based AI agent simulation framework**

**Protopolis is a project that allows you to create and simulate interactions between AI agents in a colorful terminal environment. Agents can communicate with each other and with the user according to different states and energy levels.**

[![Build Status](https://img.shields.io/github/actions/workflow/status/LightInn/protopolis/release.yml?style=for-the-badge)](https://github.com/LightInn/protopolis/actions)
[![Crates.io](https://img.shields.io/crates/v/protopolis?style=for-the-badge)](https://crates.io/crates/protopolis)
[![License](https://img.shields.io/badge/license-MIT-blue?style=for-the-badge)](https://github.com/LightInn/protopolis)
[![Downloads](https://img.shields.io/crates/d/protopolis?style=for-the-badge)](https://crates.io/crates/protopolis)

<p align="center"> 
  <img src="showcase/protopolis.gif" width="800" alt="Agents discussing philosophy"> 
</p> 

## 📋 Features

- **🧠 Agent Simulation** with different states (Idle, Thinking, Speaking)
- **💬 Messaging System** between agents and with the user
- **🌈 Colorful Terminal Interface** for better visualization
- **⚡ Energy Management** for agents
- **🔄 Simple Commands** to control the simulation

## 🛠️ Installation

```bash
git clone https://github.com/LightInn/protopolis
cd protopolis
cargo build --release
```

## 🎮 Usage

To start the simulation:

```bash
cargo run --release
```

### Available Commands

- `start` - Start the simulation
- `pause` - Pause the simulation
- `resume` - Resume the simulation
- `stop` - Stop the simulation
- `exit` - Exit the application
- `topic <subject>` - Set a new discussion topic
- `msg <agent> <message>` - Send a message to a specific agent

## 🏗️ Architecture

The project is built around several key components:

- **Agents** - Entities with states and behaviors
- **Messaging System** - Asynchronous communication between agents
- **User Interface** - Colorful display in the terminal
- **Simulation** - Orchestration of interactions

## 🗺️ Roadmap

- [x] Basic terminal interface
- [x] Color system for agents
- [x] Communication between agents
- [ ] More complex agent personalities
- [ ] Save/load simulations
- [ ] Graphical visualization of interactions

## 🤝 Contribution

Contributions are welcome! Feel free to open an issue or a pull request.

## 📚 Inspired By

- [TyniTroupe](https://github.com/microsoft/TinyTroupe) - LLM-powered multiagent persona simulation

## 🔌 Powered By

<p align="center">
  <img src="https://ollama.ai/public/ollama.png" width="200" alt="Ollama">
  <br>
  <a href="https://ollama.ai">Ollama</a> - Local LLM runner
</p>

## License

MIT © 2025 Breval LE FLOCH