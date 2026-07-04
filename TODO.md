Voici une analyse détaillée de ton implémentation actuelle et un plan d'architecture pour concevoir ton émulateur Amstrad CPC sans casser ta compatibilité TRS-80.

---

### 1. Analyse de l'existant : Gestion des opcodes `IN` / `OUT`

Actuellement, **ton CPU ne gère pas du tout les opcodes d'E/S (I/O)**. 

* Dans `cpu.rs` (`execute_1byte`), les opcodes principaux `0xDB` (`IN A, (n)`) et `0xD3` (`OUT (n), A`) ne sont pas implémentés. Ils tombent dans la branche par défaut `_` (Unknown instruction) et retournent `cycles = 0xFF`.
* Dans `execute_2bytes`, aucune instruction d'I/O du groupe d'instructions `0xED` n'est gérée (comme `IN r, (C)`, `OUT (C), r`, ou les transferts par blocs `INI`, `IND`, `OTIR`, etc.).
* Il n'existe actuellement aucune méthode ou interface pour interagir avec un espace d'I/O sur ton `Bus`. Tout passe par des accès mémoire (`read_byte`/`write_byte`).

#### 💡 Point d'attention crucial pour le CPC (décodage I/O sur 16 bits)
En Z80, même si l'instruction `IN A, (n)` ne spécifie qu'un port sur 8 bits `n`, le CPU place en réalité **le contenu du registre `A` sur les bits de poids fort (A8-A15)** du bus d'adresse pendant le cycle d'I/O. 
L'Amstrad CPC exploite massivement ce comportement pour décoder ses périphériques (par exemple, le Gate Array est sélectionné lorsque le bit 15 est à `0` et le bit 14 est à `1`, ce qui correspond souvent au port `0x7F00` en plaçant `0x7F` dans `A`). Il est donc impératif que tes futures fonctions d'E/S prennent un port d'adresse sur **16 bits** (`u16`) et non sur 8 bits.

---

### 2. Comment transformer la structure `Bus` en un Trait ?

Pour implémenter le Memory Banking lourd du CPC et le routage d'I/O, tout en gardant l'actuel TRS-80 fonctionnel, la solution idiomatique en Rust consiste à définir un **Trait `Bus`** et à utiliser des **types génériques** dans ton CPU pour conserver des performances maximales (sans surcoût d'allocation dynamique `dyn` ou de vtable).

Voici le plan d'action :

#### Étape A : Définir le Trait `Bus` (dans `bus.rs`)
Nous pouvons extraire l'interface d'accès de ton bus. Nous fournissons des implémentations par défaut pour les mots (16/32 bits) et les ports d'I/O pour alléger le travail des implémentations simples.

```rust
// Dans src/bus.rs

pub trait Bus {
    // Lecture / Écriture mémoire obligatoires
    fn read_byte(&self, address: u16) -> u8;
    fn write_byte(&mut self, address: u16, data: u8);

    // Méthodes mémoire utilitaires avec implémentations par défaut (Z80 Little Endian)
    fn read_word(&self, address: u16) -> u16 {
        u16::from(self.read_byte(address)) 
            | (u16::from(self.read_byte(address.wrapping_add(1))) << 8)
    }

    fn read_le_word(&self, address: u16) -> u16 {
        (u16::from(self.read_byte(address)) << 8) 
            | u16::from(self.read_byte(address.wrapping_add(1)))
    }

    fn read_le_dword(&self, address: u16) -> u32 {
        u32::from(self.read_byte(address)) << 24
            | u32::from(self.read_byte(address.wrapping_add(1))) << 16
            | u32::from(self.read_byte(address.wrapping_add(2))) << 8
            | u32::from(self.read_byte(address.wrapping_add(3)))
    }

    fn write_word(&mut self, address: u16, data: u16) {
        self.write_byte(address, (data & 0xFF) as u8);
        self.write_byte(address.wrapping_add(1), (data >> 8) as u8);
    }

    // Gestion de l'espace d'I/O (E/S) - 16 bits d'adresse de port
    fn read_io(&self, _port: u16) -> u8 {
        0xFF // Valeur par défaut (bus flottant)
    }

    fn write_io(&mut self, _port: u16, _data: u8) {
        // Par défaut, ne fait rien (utile pour les systèmes sans I/O)
    }
}
```

#### Étape B : Adapter ton ancienne structure `Bus` (en `SimpleBus`)
Pour ne pas casser ton TRS-80 et tes tests, renomme ta structure actuelle `Bus` en `SimpleBus` (ou conserve `Bus` mais la plupart des développeurs préfèrent renommer le Trait en `Bus` et la structure par défaut en `SimpleBus` ou `FlatBus`).

```rust
// Dans src/bus.rs

pub struct SimpleBus {
    address_space: Vec<u8>,
    rom_space: Option<ROMSpace>,
}

struct ROMSpace {
    pub start: u16,
    pub end: u16,
}

impl SimpleBus {
    pub fn new(size: u16) -> SimpleBus {
        SimpleBus {
            address_space: vec![0; (size as usize) + 1],
            rom_space: None,
        }
    }
    
    pub fn set_romspace(&mut self, start: u16, end: u16) {
        self.rom_space = Some(ROMSpace { start, end });
    }

    pub fn load_bin(&mut self, file: &str, org: u16) -> std::io::Result<usize> {
        // ... garde ton implémentation de chargement binaire ...
    }
}

// Implémentation du trait Bus pour ton SimpleBus
impl Bus for SimpleBus {
    fn read_byte(&self, address: u16) -> u8 {
        if address as usize >= self.address_space.len() {
            return 0;
        }
        self.address_space[usize::from(address)]
    }

    fn write_byte(&mut self, address: u16, data: u8) {
        if address as usize >= self.address_space.len() {
            return;
        }
        if self.rom_space.is_some()
            && address >= self.rom_space.as_ref().unwrap().start
            && address <= self.rom_space.as_ref().unwrap().end
        {
            return;
        }
        self.address_space[usize::from(address)] = data;
    }
}
```

#### Étape C : Adapter ton CPU (dans `cpu.rs`)
Tu modifies les signatures pour qu'elles acceptent n'importe quel type `B` implémentant `Bus` :

```rust
// Dans src/cpu.rs

impl CPU {
    // Utilisation d'un type générique B implémentant Bus
    pub fn execute<B: Bus>(&mut self, bus: &mut B) -> u32 {
        // ...
    }

    pub fn execute_timed<B: Bus>(&mut self, bus: &mut B) -> Option<u32> {
        // ...
    }
    
    // ... toutes les autres sous-méthodes d'exécution nécessitant le bus ...
}
```
*Note : Grâce au polymorphisme statique (monomorphisation) de Rust, le compilateur générera une version optimisée d' `execute` pour `SimpleBus` et une autre pour ton futur `CpcBus`. Aucune perte de performance !*

---

### 3. Exemple d'implémentation du CPU pour gérer `IN` / `OUT`

Maintenant que ton CPU a accès aux méthodes `read_io` et `write_io` du Trait `Bus`, tu peux implémenter les instructions d'E/S dans `cpu.rs`. 

Voici comment écrire les opcodes `0xD3` (OUT) et `0xDB` (IN) dans `execute_1byte` :

```rust
// Dans cpu.rs, match opcode de execute_1byte :

            // OUT (n), A (Opcode 0xD3)
            0xD3 => {
                let n = bus.read_byte(self.reg.pc + 1);
                // Le port d'I/O Z80 sur 16 bits : A sur le poids fort, n sur le poids faible.
                let port = ((self.reg.a as u16) << 8) | (n as u16);
                bus.write_io(port, self.reg.a);
            }

            // IN A, (n) (Opcode 0xDB)
            0xDB => {
                let n = bus.read_byte(self.reg.pc + 1);
                let port = ((self.reg.a as u16) << 8) | (n as u16);
                self.reg.a = bus.read_io(port);
            }
```

---

### 4. Projection : Comment modéliser ton futur `CpcBus` ?

Pour l'Amstrad CPC, tu pourras alors créer une structure `CpcBus` dans ton nouveau projet en réutilisant ta bibliothèque de CPU. Voici un aperçu de la structure que tu pourras concevoir pour intercepter le Banking et router les ports I/O :

```rust
pub struct CpcBus {
    pub ram: Vec<u8>,        // 64 Ko ou 128 Ko de RAM physique
    pub os_rom: Vec<u8>,     // 16 Ko (mappée en bas de 0x0000 à 0x3FFF)
    pub basic_rom: Vec<u8>,  // 16 Ko (mappée en haut de 0xC000 à 0xFFFF)
    
    // État du Banking configuré par le Gate Array
    pub lower_rom_enabled: bool,
    pub upper_rom_enabled: bool,
    pub selected_upper_rom: u8, // 0 = BASIC, 7 = AMSDOS (Lecteur disquette)...
    
    // Périphériques simulés
    // pub gate_array: GateArray,
    // pub crtc: Crtc,
    // pub ppi: Intel8255,
}

impl Bus for CpcBus {
    fn read_byte(&self, address: u16) -> u8 {
        match address {
            // Bloc 0 : 0x0000 - 0x3FFF (RAM ou OS ROM)
            0x0000..=0x3FFF => {
                if self.lower_rom_enabled {
                    self.os_rom[address as usize]
                } else {
                    self.ram[address as usize]
                }
            }
            // Blocs 1 & 2 : 0x4000 - 0xBFFF (Toujours de la RAM)
            0x4000..=0xBFFF => {
                self.ram[address as usize]
            }
            // Bloc 3 : 0xC000 - 0xFFFF (RAM ou BASIC ROM)
            0xC000..=0xFFFF => {
                if self.upper_rom_enabled && self.selected_upper_rom == 0 {
                    self.basic_rom[(address - 0xC000) as usize]
                } else {
                    self.ram[address as usize]
                }
            }
        }
    }

    fn write_byte(&mut self, address: u16, data: u8) {
        // Particularité Amstrad CPC : Les écritures en mémoire ignorent le mapping ROM
        // et écrivent systématiquement dans la RAM physique sous-jacente.
        self.ram[address as usize] = data;
    }

    fn write_io(&mut self, port: u16, data: u8) {
        // Routage I/O CPC via décodage partiel d'adresse
        
        // 1. Gate Array (gère le Banking et la palette vidéo)
        // Sélectionné si le bit 15 est à 0 et le bit 14 est à 1 (port & 0xC000 == 0x4000)
        if (port & 0xC000) == 0x4000 {
            // Ici tu parses l'octet 'data' pour changer lower_rom_enabled / upper_rom_enabled
            // self.gate_array.write(data, self);
        }
        
        // 2. CRTC (gère l'affichage)
        // Sélectionné si le bit 14 est à 0 (port & 0x4000 == 0)
        else if (port & 0x4000) == 0 {
            // self.crtc.write(port, data);
        }
        
        // 3. PPI Intel 8255 (Clavier, AY-3-8910 Audio/Joystick, Cassette...)
        // Sélectionné si le bit 11 est à 0 (port & 0x0800 == 0)
        else if (port & 0x0800) == 0 {
            // self.ppi.write(port & 3, data);
        }
    }

    fn read_io(&self, port: u16) -> u8 {
        // Même logique de décodage partiel pour la lecture
        if (port & 0x4000) == 0 {
            // return self.crtc.read();
            0xFF
        } else if (port & 0x0800) == 0 {
            // return self.ppi.read(port & 3);
            0xFF
        } else {
            0xFF
        }
    }
}
```
