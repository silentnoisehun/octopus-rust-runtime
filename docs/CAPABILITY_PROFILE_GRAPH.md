# Capability profile AwareGraph

## Viselkedési cél

A runtime külön jelentse, hogy egy capability elérhető-e, milyen hatást végez, és milyen erősen igazolt. A Technical Marshal csak a `windows-offline` profilban engedélyezett, legalább automatizáltan tesztelt jelöltek között végezhet ψ-súlyozott választást.

## Gráf

| Forráscsomópont | Él | Célcsomópont | Jelenlegi / kívánt szerződés |
|---|---|---|---|
| `lib::list()` | összefűz | `blade::list()` + runtime-only adapters + Bio subsystem | 225 egyedi publikus név, köztük 33 külön folyamatú Bio-Binaries target |
| `capability::classify()` | ad | mode + status | platform- és elérhetőségi tengely megmarad |
| mode + capability name | származtat | execution class | advisory, local-operation, external-integration vagy control-plane |
| status + adaptertesztek | származtat | verification grade | declared, tested vagy observed |
| `CapabilityProfile::WindowsOffline` | szűr | capability catalog | csak `real`, nem külső, legalább `tested` |
| `render_capabilities*()` | materializál | CLI/MCP | az új tengelyek láthatóak és auditálhatóak |
| `marshal::ready_candidates()` | olvas | Windows/offline profil | a puszta `status == real` többé nem elég |
| Marshal receipt | dokumentál | terminál/log | profil és minimum ellenőrzési fokozat látható |
| unit + integration tests | validál | registry/CLI/Marshal | 225-ös all-lista és 164-es offline profilhatár egyaránt bizonyított |
| `RELEASE_SHA256SUMS` + process adapter | kapuz | 33 Bio executable | eltérő hash esetén indulás előtti typed failure |
| README + capability matrix | dokumentál | felhasználói szerződés | nem állítja, hogy minden `real` külső művelet |

## Legkisebb vágás

1. `src/capability.rs`: új execution-class, verification-grade és profile típusok; katalogizálás és renderelés.
2. `src/lib.rs`: profilozott publikus lekérdezés és re-export.
3. `src/marshal.rs`: `windows-offline` + `tested` kapu a ψ-választás előtt.
4. `src/main.rs`: `capabilities --profile all|windows-offline`.
5. Tesztek és dokumentáció: az új élek ellenőrzése.

## Megőrzendő invariánsok

- A teljes registry 225 egyedi név.
- A `status`, az execution class és a verification grade külön tengely.
- Az `apple-notes` és `bear-notes` továbbra is typed `unsupported` hibát ad.
- A ψ-választás nem változtat engedélyen, platformon vagy ellenőrzési fokozaton.
- Írást végző Marshal-topológia továbbra is csak `--execute --allow-write` mellett indulhat.
- A telepített v2.8.0 endurance-bináris a 24 órás futás alatt nem cserélhető le.
