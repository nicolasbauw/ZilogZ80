# Reprise : drapeaux non documentés XF/YF (branche `z80-drapeaux-non-documentes`)

**État : terminé. Les 325 tests passent, la branche est verte.**

## Le problème traité

Les deux drapeaux non documentés du Z80 (bits 3 et 5 du registre F, dits
XF et YF) n'étaient **jamais calculés** : les champs `b3`/`b5` existaient
dans `Flags` et étaient bien sérialisés par `to_byte`/`from_byte`, mais
aucune instruction ne les positionnait. Ils restaient donc à zéro en
permanence, sauf restauration par `POP AF`.

Ces bits n'ont pas de signification propre — le Z80 y laisse simplement
transparaître les bits correspondants de sa dernière opération. Ils ne
sont observables qu'à travers `PUSH AF`, mais certaines protections de
copie s'en servent précisément pour ça.

## Ce qui est implémenté

Deux primitives dans `src/flags.rs` :

- `set_undocumented_from(value)` : XF = bit 3, YF = bit 5 (cas général) ;
- `set_undocumented_from_block(value)` : XF = bit 3, YF = **bit 1**,
  règle propre aux instructions de bloc.

Appliquées avec la source correcte pour chaque famille :

| famille | source des bits |
|---|---|
| `ADD`/`ADC`/`SUB`/`SBC`/`AND`/`OR`/`XOR`/`INC`/`DEC`/`NEG`/`DAA` | le résultat |
| rotations et décalages (`RLC`…`SRL`, `SLL`) | le résultat |
| **`CP`** | l'**opérande** (exception classique : c'est ce qui le distingue de `SUB` à l'exécution) |
| `ADD`/`ADC`/`SBC HL,rr` | octet de **poids fort** du résultat |
| `LDI`/`LDD` (et formes répétitives) | `A + octet transféré`, règle de bloc |
| `CPI`/`CPD` (et formes répétitives) | `A - (HL) - demi-report`, règle de bloc |
| `BIT` | la valeur testée |
| `IN r,(C)` et `IN F,(C)` | l'octet lu |
| `SCF`/`CCF` | `A` (comportement Zilog NMOS, celui du CPC) |
| `RLD`/`RRD`, `LD A,I`, `LD A,R` | le résultat / la valeur chargée |
| `CPL` | le résultat (le `A` complémenté) |
| `RLCA`/`RRCA`/`RLA`/`RRA` | le résultat |
| E/S par bloc (`INI`/`IND`/`OUTI`/`OUTD`…) | `B` **après** décrémentation |

**Deux trous supplémentaires corrigés au passage** : `BIT` ne posait ni
`S` ni `P/V`, alors que le Z80 pose `S` quand on teste le bit 7 et qu'il
vaut 1, et recopie `Z` dans `P/V`.

## Preuve que l'implémentation est correcte

Vérifiée sur un cas concret plutôt que supposée. Dans
`adc_a_ixh_ixl_asm`, après `ADC A,IXL` on a `A = 0xA2` = `1010_0010` :
bit 3 à 0, bit 5 à 1. Le drapeau correct est donc `SF|VF|YF` = `0xA4`
(164). Le code produit exactement cette valeur ; l'attente historique de
`0x84` (132) était incomplète, faute de XF/YF.

## Trous trouvés et bouchés pendant le recalage

L'audit des tests a mis au jour trois familles que le premier passage avait
oubliées, et qui ne posaient donc toujours pas XF/YF :

- **`CPL`** : les deux bits viennent du résultat (le `A` complémenté) ;
- **`RLCA`/`RRCA`/`RLA`/`RRA`** : idem, le résultat — seules les rotations
  préfixées `CB` avaient été traitées, pas leurs quatre variantes courtes
  sur l'accumulateur ;
- **instructions d'E/S par bloc** (`INI`/`IND`/`OUTI`/`OUTD` et leurs
  formes répétitives) : XF/YF viennent de `B` **après** décrémentation,
  comme SF et ZF.

## Recalage des tests

Les 46 tests ont été repris assertion par assertion, en dérivant XF/YF de
la valeur source correcte selon le tableau ci-dessus — jamais en
recopiant ce que produit le code. La valeur source est presque toujours
déjà affirmée par le test juste au-dessus de l'assertion de drapeaux ;
là où elle ne l'est pas (`CP`, instructions de bloc, `RLD`/`RRD`), elle a
été retrouvée dans le binaire de test et notée en commentaire.

Les constantes `XF`/`YF` sont désormais actives en tête de `src/test.rs`.

Un test dédié, `undocumented_flags_rotations_cpl_block_io`, couvre les
trois familles ci-dessus : le reste de la suite ne les exerçait sur
aucune valeur ayant à la fois le bit 3 et le bit 5, si bien qu'un oubli
n'y aurait rien cassé.

## Drapeaux documentés des E/S par bloc

Traité depuis, dans la foulée. `INI`/`IND`/`OUTI`/`OUTD` et leurs formes
répétitives ne posaient ni S, ni H, ni P/V, et mettaient N à 1
systématiquement. Les règles réelles :

- S, Z (et XF/YF) viennent de `B` **après** décrémentation ;
- `N` recopie le **bit 7 de l'octet transféré** — seul cas de la machine
  où ce drapeau ne vaut pas 1 après une opération qui le pose ;
- H, C et P/V se déduisent d'une somme intermédiaire `k` entre cet octet
  et un second terme propre à la famille (`C + 1` pour `INI`, `C - 1`
  pour `IND`, `L` après mise à jour pour les sorties) : H et C valent le
  débordement de `k`, et P/V la parité de `(k & 7) ^ B`.

Les huit sites partagent le helper `CPU::block_io_flags`. Couvert par
`block_io_documented_flags`.

## MEMPTR/WZ

Traité également. Le registre est modélisé (`Registers::wz`) et mis à
jour par toutes les familles concernées : accès indexés, chargements vers
et depuis `(nn)`, `EX (SP),rr`, arithmétique 16 bits, sauts absolus
(pris ou non) et relatifs (pris seulement), `CALL`, `RET`, `RST`,
interruptions, E/S directes et par bloc, et les formes répétitives de
transfert et de comparaison.

Deux règles sortent du lot :

- les quatre instructions qui présentent `A` sur la moitié haute du bus
  d'adresse (`LD (nn),A`, `LD (BC),A`, `LD (DE),A`, `OUT (n),A`)
  n'avancent que l'octet de poids faible et chargent `A` dans le poids
  fort (`CPU::wz_after_write_a`) ;
- `LDIR`/`LDDR`/`CPIR`/`CPDR` qui se répètent rechargent MEMPTR avec
  l'adresse de leur propre opcode plus un, le processeur s'apprêtant à le
  relire (`CPU::repeat_block_wz`). Les formes répétitives d'E/S par bloc
  ne suivent **pas** cette règle.

`BIT b,(HL)` et `BIT b,(IX+d)` prennent désormais leurs deux drapeaux non
documentés sur le poids fort de MEMPTR. La forme indexée en a profité :
elle ne posait ni S, ni P/V, ni ces deux bits.

Couvert par `memptr_update_rules` (26 cas) et
`bit_undocumented_flags_come_from_memptr`.

L'implémentation de ces règles s'est appuyée sur une refonte préalable de
l'adressage indexé : les 50 accès `(IX+d)`/`(IY+d)` répétaient chacun un
`if/else` distinguant déplacement positif et négatif, là où une extension
de signe suffit. Ils passent par `ix_d`/`iy_d`, qui sont aussi le point
d'accroche de MEMPTR.

## Validation

`bin/zexall.com` passe ses 67 tests, sans erreur. C'est le banc qui
contrôle **tous** les bits du registre F, drapeaux non documentés
compris — contrairement à `zexdoc.com`, qui les masque et ne validait
donc rien de ce travail-ci. Les lignes qui comptent ici sont
`bit n,<b,c,d,e,h,l,(hl),a>` et `bit n,(<ix,iy>+1)` pour MEMPTR,
`<daa,cpl,scf,ccf>` et `<rlca,rrca,rla,rra>` pour les trous bouchés au
recalage, et les quatre lignes `cpi<r>`/`cpd<r>`/`ldi<r>`/`ldd<r>` pour
la règle de bloc.

Ce résultat a été soumis à un contrôle négatif : en rétablissant
l'ancienne approximation de `BIT b,(HL)` (les deux bits pris sur la
valeur lue au lieu du poids fort de MEMPTR), zexall signale bien
`bit n,<b,c,d,e,h,l,(hl),a>` en erreur, CRC à l'appui. Le « OK » n'est
donc pas vide de sens.

Une réserve subsiste malgré tout : zexall ne teste pas les instructions
d'E/S ni les interruptions. Les règles MEMPTR de `IN`/`OUT`, des E/S par
bloc et de l'acquittement d'interruption, ainsi que les drapeaux
documentés des E/S par bloc, ne reposent donc que sur les tests écrits
ici, dérivés à la main des règles publiées.
