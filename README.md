# 🤖 Protopolis

**Un framework de simulation d'agents IA en Rust**

**Protopolis est un projet permettant de créer et de simuler des interactions entre agents IA dans un environnement
terminal coloré. Les agents peuvent communiquer entre eux et avec l'utilisateur selon différents états et niveaux d'
énergie.**

[![Build Status](https://img.shields.io/github/actions/workflow/status/LightInn/protopolis/release.yml?style=for-the-badge)](https://github.com/LightInn/protopolis/actions)
[![Crates.io](https://img.shields.io/crates/v/protopolis?style=for-the-badge)](https://crates.io/crates/protopolis)
[![License](https://img.shields.io/badge/license-MIT-blue?style=for-the-badge)](https://github.com/LightInn/protopolis)
[![Downloads](https://img.shields.io/crates/d/protopolis?style=for-the-badge)](https://crates.io/crates/protopolis)

## 📋 Fonctionnalités

- **🧠 Simulation d'agents** avec différents états (Idle, Thinking, Speaking)
- **💬 Système de messagerie** entre agents et avec l'utilisateur
- **🌈 Interface terminal colorée** pour une meilleure visualisation
- **⚡ Gestion d'énergie** des agents
- **🔄 Commandes simples** pour contrôler la simulation

## 🛠️ Installation

```bash
git clone https://github.com/LightInn/protopolis
cd protopolis
cargo build --release
```

## 🎮 Utilisation

Pour lancer la simulation :

```bash
cargo run --release
```

### Commandes disponibles

- `start` - Démarrer la simulation
- `pause` - Mettre en pause la simulation
- `resume` - Reprendre la simulation
- `stop` - Arrêter la simulation
- `exit` - Quitter l'application
- `topic <sujet>` - Définir un nouveau sujet de discussion
- `msg <agent> <message>` - Envoyer un message à un agent spécifique

## 🏗️ Architecture

Le projet est construit autour de plusieurs composants clés :

- **Agents** - Entités avec états et comportements
- **Système de messages** - Communication asynchrone entre agents
- **Interface utilisateur** - Affichage coloré dans le terminal
- **Simulation** - Orchestration des interactions

## 🗺️ Roadmap

- [x] Interface terminal de base
- [x] Système de couleurs pour les agents
- [x] Communication entre agents
- [ ] Personnalités d'agents plus complexes
- [ ] Sauvegarde/chargement de simulations
- [ ] Visualisation graphique des interactions

## 🤝 Contribution

Les contributions sont les bienvenues ! N'hésitez pas à ouvrir une issue ou une pull request.

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













