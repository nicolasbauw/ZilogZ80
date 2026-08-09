# Reprise : drapeaux non documentés XF/YF (branche `z80-drapeaux-non-documentes`)

**État : implémentation faite et vérifiée, 46 tests existants à recaler.
La branche est volontairement rouge — ne pas fusionner telle quelle.**

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

**Deux trous supplémentaires corrigés au passage** : `BIT` ne posait ni
`S` ni `P/V`, alors que le Z80 pose `S` quand on teste le bit 7 et qu'il
vaut 1, et recopie `Z` dans `P/V`.

## Preuve que l'implémentation est correcte

Vérifiée sur un cas concret plutôt que supposée. Dans
`adc_a_ixh_ixl_asm`, après `ADC A,IXL` on a `A = 0xA2` = `1010_0010` :
bit 3 à 0, bit 5 à 1. Le drapeau correct est donc `SF|VF|YF` = `0xA4`
(164). Le code produit exactement cette valeur ; l'attente historique de
`0x84` (132) était incomplète, faute de XF/YF.

## Ce qui reste à faire

**46 tests existants échouent**, parce qu'ils figent des octets de
drapeaux calculés à une époque où XF/YF valaient toujours zéro.

⚠️ **Piège à éviter absolument** : ne PAS recopier mécaniquement ce que
produit le code dans les attentes — le test ne prouverait plus rien. Pour
chaque assertion, dériver XF/YF de la **valeur source correcte selon le
tableau ci-dessus**, en s'appuyant sur la valeur de registre que le test
affirme juste avant (la plupart le font, cf. l'exemple `0xA2` plus haut).
Attention aux familles dont la source n'est pas le résultat : `CP`,
`BIT`, les instructions de bloc, les opérations 16 bits.

Ajouter au passage des constantes `XF`/`YF` à côté des `CF`/`NF`/`VF`
existantes en tête de `src/test.rs`, pour que les attentes restent
lisibles.

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
À traiter séparément, une fois cette branche terminée et fusionnée.
