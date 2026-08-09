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

## Reste à faire, hors périmètre de cette branche

Les instructions d'E/S par bloc ne posent toujours pas leurs drapeaux
**documentés** (S, H, P/V) — seuls Z, N et maintenant XF/YF le sont.
C'est un manque antérieur à cette branche, et qui ne relève pas des
drapeaux non documentés ; le test dédié le constate explicitement.

## Approximation connue, non traitée : MEMPTR/WZ

`BIT b,(HL)` devrait prendre ses deux bits sur l'octet de poids fort du
registre interne **MEMPTR** (aussi appelé WZ), et non sur la valeur lue.
C'est actuellement approximé par la valeur lue, avec un commentaire dans
`Cpu::bit`.

Implémenter MEMPTR proprement est un chantier à part : ce registre est
mis à jour par des dizaines d'instructions (`JP`, `CALL`, `RET`, `RST`,
`DJNZ` pris, `LD A,(nn)`, `LD (nn),A`, `EX (SP),HL`, `ADD/ADC/SBC HL`,
accès indexés `(IX+d)`, `IN`/`OUT`, instructions de bloc en répétition…),
pour une seule manifestation observable : les deux bits de `BIT b,(HL)`.
À traiter séparément, une fois cette branche fusionnée.
