# Essai de la chaîne de mise à jour sous Windows (pari 4), pendant de essai-maj.sh :
# installe N avec l'installeur Velopack, puis N → N+1 (paquet complet) et N+1 → N+2 (delta)
# par le manifeste signé, et refuse un manifeste falsifié. Demande vpk. Depuis rust/.
$ErrorActionPreference = 'Stop'
$T = Join-Path $env:RUNNER_TEMP 'essai-maj'
if (-not $env:RUNNER_TEMP) { $T = Join-Path $env:TEMP 'essai-maj' }
Remove-Item -Recurse -Force $T -ErrorAction SilentlyContinue
New-Item -ItemType Directory -Force "$T\app", "$T\depot" | Out-Null
$env:BS_JOURNAL = "$T\journal.log"; $env:BS_FICHIER_SERIE = "$T\serie"
cargo build -q --release -p bs-maj --example demo_maj --bin bs-signer
if ($LASTEXITCODE) { throw 'build' }
$S = (Resolve-Path 'target\release\bs-signer.exe').Path
Copy-Item 'target\release\examples\demo_maj.exe' "$T\app\"
$cles = & $S cles
$env:BS_CLE_PRIVEE = ($cles[0] -split ': ')[1].Trim(); $env:BS_CLE_PUBLIQUE = ($cles[1] -split ': ')[1].Trim()

function Empaqueter($version, $serie) {
  Set-Content "$T\app\donnees.txt" "données $version"
  Push-Location $T; vpk pack -u BSDemo -v $version -p "$T\app" -e demo_maj.exe -o "$T\depot" -c stable | Out-Null; Pop-Location
  if ($LASTEXITCODE) { throw "vpk pack $version" }
  & $S manifeste "$T\depot" stable $serie 30 0.1.0 | Out-Null
}
function Attendu($texte) {
  Start-Sleep 8
  # @( ) : avec une seule ligne, Get-Content rend une chaîne et [-1] son dernier caractère
  $derniere = @(Get-Content $env:BS_JOURNAL -Encoding utf8)[-1]
  if ($derniere -like "*$texte*") { "ok    : $derniere" } else { throw "ÉCHEC : attendu « $texte », journal : $derniere" }
}
$appli = "$env:LOCALAPPDATA\BSDemo\current\demo_maj.exe"

"== installation de N puis mises à jour"
Empaqueter 0.1.0 1
& "$T\depot\BSDemo-stable-Setup.exe" --silent | Out-Null
Start-Sleep 5
& $appli; Attendu 'démarré : version 0.1.0'
Empaqueter 0.2.0 2
& $appli --maj "$T\depot"; Attendu 'démarré : version 0.2.0'
Empaqueter 0.3.0 3
& $appli --maj "$T\depot"; Attendu 'démarré : version 0.3.0'
if (Select-String -Path $env:BS_JOURNAL -Pattern '0.3.0 téléchargée \(delta\)' -Quiet) { "ok    : N+1 → N+2 par le delta" } else { "info  : N+1 → N+2 sans delta" }

"== manifeste falsifié"
Empaqueter 0.4.0 4
(Get-Content "$T\depot\version.json" -Raw) -replace '\\"taille\\": (\d)', '\"taille\": 9$1' | Set-Content "$T\depot\version.json" -NoNewline
& $appli --maj "$T\depot"; Attendu 'refusé : signature du manifeste invalide'
"== tout est bon"
