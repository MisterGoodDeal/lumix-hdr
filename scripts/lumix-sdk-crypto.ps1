# Chiffrement partagé pack CI / restore CI (AES-256-CBC, PBKDF2-SHA256, 200k iter).
# Fichier : magic ASCII "LHDR1" (5) + salt (16) + IV (16) + ciphertext.

function Protect-LumixSdkZip {
    param(
        [Parameter(Mandatory = $true)][string]$ZipPath,
        [Parameter(Mandatory = $true)][string]$EncPath,
        [Parameter(Mandatory = $true)][string]$Passphrase
    )

    $plain = [IO.File]::ReadAllBytes($ZipPath)
    $salt = New-Object byte[] 16
    $iv = New-Object byte[] 16
    $rng = [System.Security.Cryptography.RandomNumberGenerator]::Create()
    $rng.GetBytes($salt)
    $rng.GetBytes($iv)

    $pdb = New-Object System.Security.Cryptography.Rfc2898DeriveBytes(
        $Passphrase,
        $salt,
        200000,
        [System.Security.Cryptography.HashAlgorithmName]::SHA256
    )
    $key = $pdb.GetBytes(32)

    $aes = [System.Security.Cryptography.Aes]::Create()
    $aes.Mode = [System.Security.Cryptography.CipherMode]::CBC
    $aes.Padding = [System.Security.Cryptography.PaddingMode]::PKCS7
    $aes.Key = $key
    $aes.IV = $iv
    $encryptor = $aes.CreateEncryptor()
    $cipher = $encryptor.TransformFinalBlock($plain, 0, $plain.Length)

    $magic = [Text.Encoding]::ASCII.GetBytes("LHDR1")
    $out = New-Object byte[] ($magic.Length + $salt.Length + $iv.Length + $cipher.Length)
    [Array]::Copy($magic, 0, $out, 0, $magic.Length)
    [Array]::Copy($salt, 0, $out, $magic.Length, $salt.Length)
    [Array]::Copy($iv, 0, $out, $magic.Length + $salt.Length, $iv.Length)
    [Array]::Copy($cipher, 0, $out, $magic.Length + $salt.Length + $iv.Length, $cipher.Length)
    [IO.File]::WriteAllBytes($EncPath, $out)

    $aes.Dispose()
    $pdb.Dispose()
}

function Unprotect-LumixSdkZip {
    param(
        [Parameter(Mandatory = $true)][string]$EncPath,
        [Parameter(Mandatory = $true)][string]$ZipPath,
        [Parameter(Mandatory = $true)][string]$Passphrase
    )

    $raw = [IO.File]::ReadAllBytes($EncPath)
    $magic = [Text.Encoding]::ASCII.GetBytes("LHDR1")
    if ($raw.Length -lt ($magic.Length + 32) ) {
        throw "Fichier .enc trop court"
    }
    for ($i = 0; $i -lt $magic.Length; $i++) {
        if ($raw[$i] -ne $magic[$i]) {
            throw "Magic LHDR1 invalide — mauvais fichier .enc ?"
        }
    }

    $salt = New-Object byte[] 16
    $iv = New-Object byte[] 16
    [Array]::Copy($raw, 5, $salt, 0, 16)
    [Array]::Copy($raw, 21, $iv, 0, 16)
    $cipherLen = $raw.Length - 37
    $cipher = New-Object byte[] $cipherLen
    [Array]::Copy($raw, 37, $cipher, 0, $cipherLen)

    $pdb = New-Object System.Security.Cryptography.Rfc2898DeriveBytes(
        $Passphrase,
        $salt,
        200000,
        [System.Security.Cryptography.HashAlgorithmName]::SHA256
    )
    $key = $pdb.GetBytes(32)

    $aes = [System.Security.Cryptography.Aes]::Create()
    $aes.Mode = [System.Security.Cryptography.CipherMode]::CBC
    $aes.Padding = [System.Security.Cryptography.PaddingMode]::PKCS7
    $aes.Key = $key
    $aes.IV = $iv
    try {
        $decryptor = $aes.CreateDecryptor()
        $plain = $decryptor.TransformFinalBlock($cipher, 0, $cipher.Length)
    }
    catch {
        throw "Échec du déchiffrement (mauvaise passphrase ?)"
    }
    finally {
        $aes.Dispose()
        $pdb.Dispose()
    }

    [IO.File]::WriteAllBytes($ZipPath, $plain)
}
