# CI assets

`lumix-sdk-library.zip.enc` — archive **chiffrée** (AES-256-CBC / PBKDF2, format
`LHDR1`) de `vendor/lumix-sdk/Library/` (DLL + `.lib` + headers). Pas de binaire
en clair dans le dépôt.

Régénérer :

```powershell
.\scripts\pack-lumix-sdk.ps1
```

Secret GitHub associé : `LUMIX_SDK_PASSPHRASE` (voir `docs/release.md`).
