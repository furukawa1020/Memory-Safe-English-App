# Mobile アプリ

`apps/mobile` は Memory Safe English の Flutter クライアントです。

このアプリは、英語をそのまま大量に見せるのではなく、ワーキングメモリ負荷を下げる表示と導線を優先しています。

## 今できること

- メール認証ベースのログイン導線
- コンテンツ一覧と Reader 画面
- `Normal / Chunk / Skeleton / Assisted` の切り替え
- `reader-plan` を使った focus step 表示
- `listening-plan` と `speaking-plan` の分析表示
- proxy の `/bootstrap/mobile` と `/ready` を使った起動確認

## 前提

- Flutter SDK が使えること
- Android SDK と emulator / adb が使えること
- backend stack が `http://127.0.0.1:8070` の Rust proxy 経由で起動していること

Flutter が `PATH` に無くても、スクリプトへ `-FlutterPath` を渡せます。
深いパスでも大丈夫で、たとえば次のような指定から自動で SDK root を解決します。

```powershell
-FlutterPath "C:\Users\hatake\Downloads\flutter_windows_3.38.5-stable\flutter\packages\flutter_tools\gradle\build\classes\kotlin\main\com\flutter"
```

毎回長いパスを渡したくない場合は、一度だけ保存できます。

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\save-mobile-config.ps1 `
  -FlutterPath "C:\Users\hatake\Downloads\flutter_windows_3.38.5-stable\flutter\packages\flutter_tools\gradle\build\classes\kotlin\main\com\flutter" `
  -AndroidSdkRoot "C:\Users\hatake\AppData\Local\Android\Sdk"
```

これで以後は `-FlutterPath` や `-AndroidSdkRoot` を省略できます。

## まず確認

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\dev-doctor.ps1 `
  -FlutterPath "C:\Users\hatake\Downloads\flutter_windows_3.38.5-stable\flutter\packages\flutter_tools\gradle\build\classes\kotlin\main\com\flutter"
```

## bootstrap

Android だけ先に整える場合:

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\bootstrap-mobile.ps1 `
  -AndroidOnly `
  -SkipPubGet `
  -FlutterPath "C:\Users\hatake\Downloads\flutter_windows_3.38.5-stable\flutter\packages\flutter_tools\gradle\build\classes\kotlin\main\com\flutter"
```

これで次を実行します。

- `flutter create . --platforms android`
- 必要なら `flutter pub get`

初回の Flutter 環境によっては `pub get` がかなり時間がかかるので、まず platform 生成だけ先に通したいときは `-SkipPubGet` を付けると切り分けしやすいです。

## エミュレーター起動

system image や emulator package が足りない場合は、先にこれを実行できます。

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\install-android-emulator-deps.ps1 `
  -AndroidSdkRoot "C:\Program Files\Unity\Hub\Editor\6000.0.66f2\Editor\Data\PlaybackEngines\AndroidPlayer\SDK"
```

`Program Files` 配下の SDK へ直接書き込めない場合は、書き込み可能な SDK root を repo 内へ作る方法もあります。

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\provision-user-android-sdk.ps1 `
  -SourceAndroidSdkRoot "C:\Program Files\Unity\Hub\Editor\6000.0.66f2\Editor\Data\PlaybackEngines\AndroidPlayer\SDK" `
  -DestinationAndroidSdkRoot ".android-sdk" `
  -SaveConfig
```

これで `.android-sdk` に `platform-tools / emulator / system image` を入れ、以後のスクリプトもその SDK root を使いやすくなります。

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\start-android-emulator.ps1
```

Android SDK が `PATH` に無い場合は `-AndroidSdkRoot` を渡せます。

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\start-android-emulator.ps1 `
  -AndroidSdkRoot "C:\Users\hatake\AppData\Local\Android\Sdk"
```

## アプリ起動

最小の実行コマンド:

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\run-mobile.ps1 `
  -FlutterPath "C:\Users\hatake\Downloads\flutter_windows_3.38.5-stable\flutter\packages\flutter_tools\gradle\build\classes\kotlin\main\com\flutter"
```

backend と emulator もまとめて起動したい場合:

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\run-mobile.ps1 `
  -StartStack `
  -StartEmulator `
  -FlutterPath "C:\Users\hatake\Downloads\flutter_windows_3.38.5-stable\flutter\packages\flutter_tools\gradle\build\classes\kotlin\main\com\flutter"
```

内部では次と同等です。

```bash
flutter run --dart-define=API_BASE_URL=http://10.0.2.2:8070
```

`10.0.2.2` は Android emulator からホスト PC へ戻るためのアドレスです。

## 主な接続先

mobile は Go API を直接叩かず、Rust proxy を入口に使います。

- `/ready`
- `/bootstrap/mobile`
- `/auth/login`
- `/auth/refresh`
- `/contents`
- `/analysis/chunks`
- `/analysis/reader-plan`
- `/analysis/listening-plan`
- `/analysis/speaking-plan`

## 補足

- `android/` が無い場合でも `bootstrap-mobile.ps1` が自動で生成できます
- 起動前に `dev-doctor.ps1` を通すと、足りないのが `Docker / Flutter / adb / emulator / AVD` のどれかすぐ分かります
