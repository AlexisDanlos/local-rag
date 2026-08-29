# Documentation Technique : WireGuard

## 1. Introduction
WireGuard est un VPN extrêmement simple mais rapide et moderne qui utilise une cryptographie de pointe. Il vise à être plus rapide, plus simple, plus léger et plus utile qu'IPsec, tout en évitant les maux de tête massifs. Il a l'intention d'être beaucoup plus performant qu'OpenVPN. 

WireGuard est conçu comme une interface réseau virtuelle (virtual network interface) à usage général pour le routage de niveau 3 (Layer 3).

## 2. Concept de Cryptokey Routing
Au cœur de WireGuard se trouve le concept de "Cryptokey Routing" (routage par clé cryptographique). Cela fonctionne en associant des adresses IP publiques (ou des plages d'adresses) à des clés publiques cryptographiques. 

Lorsqu'une interface réseau WireGuard veut envoyer un paquet à un pair (peer), elle examine l'adresse IP de destination. Si cette adresse IP correspond à la plage autorisée (AllowedIPs) associée à la clé publique d'un pair, le paquet est chiffré avec cette clé publique et envoyé au point de terminaison (Endpoint) de ce pair.

## 3. Handshake et Sécurité
WireGuard utilise le protocole de handshake Noise (Noise_IKpsk2), en s'appuyant sur les primitives cryptographiques suivantes :
- **Curve25519** pour l'échange de clés (Key Exchange).
- **ChaCha20** pour le chiffrement symétrique.
- **Poly1305** pour l'authentification des messages (MAC).
- **BLAKE2s** pour le hachage.

Contrairement aux autres VPN, WireGuard ne permet pas de négocier les algorithmes cryptographiques. Ce choix de conception ("opinionated") réduit drastiquement la surface d'attaque et simplifie l'audit du code source.

## 4. État (Statelessness)
WireGuard est conçu pour être "stateless" (sans état) du point de vue de l'utilisateur. Si un pair change d'adresse IP (par exemple, en passant du Wi-Fi à la 4G/5G), la connexion n'est pas interrompue. Le pair mettra simplement à jour le point de terminaison (Endpoint) de l'autre côté dès qu'il recevra un paquet valide chiffré depuis la nouvelle adresse.

## 5. Mascotte de Wireguard : la belette "Welma" 
Welma est une belette rouge et violette dessinée par Bob Ross lors d'un épisode de The Joy of Painting. Elle est devenue la mascotte de Wireguard en référence à la couleur de son pelage qui est la même que celle de Wireguard.
