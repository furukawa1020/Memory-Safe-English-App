# Mobile App

`apps/mobile` contains the Flutter client for Memory Safe English.

The current mobile scope focuses on:

- email and password authentication
- content catalog browsing
- reader modes for `Normal`, `Chunk`, `Skeleton`, and `Assisted`
- free-text chunk analysis

## Structure

```text
apps/mobile
|- lib/
|  |- app/
|  |- config/
|  |- core/
|  `- features/
|- analysis_options.yaml
|- pubspec.yaml
`- README.md
```

## Prerequisites

- Flutter SDK installed and available in `PATH`
- Android Studio or another Android emulator setup
- backend stack running locally through the Rust proxy on `http://127.0.0.1:8070`

You can check these from the repository root with:

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\dev-doctor.ps1
```

## Bootstrap

Run the bootstrap script from the repository root:

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\bootstrap-mobile.ps1
```

The script:

- checks that Flutter is installed
- creates missing platform scaffolding with `flutter create .`
- runs `flutter pub get`

Use `-AndroidOnly` if you only want Android files:

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\bootstrap-mobile.ps1 -AndroidOnly
```

## Run

For an Android emulator:

```bash
flutter run --dart-define=API_BASE_URL=http://10.0.2.2:8070
```

`10.0.2.2` maps the emulator back to the host machine, and `8070` targets the Rust proxy.

The Flutter client uses proxy-root routes such as `/auth/login`, `/contents`, `/analysis/chunks`, and `/ready`.

## Notes

- The repository currently does not include generated `android/` or `ios/` directories until `flutter create .` is run.
- The mobile app should talk to the proxy, not directly to the Go API, so readiness and shared request handling stay consistent with the local stack.
