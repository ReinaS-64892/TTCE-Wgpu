# TTCE-Wgpu

## これは?

TexTransTool の Core で使用される TexTransCoreEngine の Wgpu 実装です。

## 開発環境のセットアップ

### dotnet を環境に install する

[参考](https://dotnet.microsoft.com/download)

現在、主に バージョンは `.NET 8.0` が使用されていますが、いずれバージョンが上がる可能性があります。

### Rust , Cargo を環境にインストールする

[参照](https://www.rust-lang.org/tools/install)

### DirectXShaderCompiler を置く

私が適当に作った [DXC-Binary](https://github.com/ReinaS-64892/DXC-Binary/releases/latest) か [DirectXShaderCompiler(Official)](https://github.com/microsoft/DirectXShaderCompiler/releases/latest) のページからバイナリをダウンロードし、 "dxcompiler_build" に直接展開してください。

example: `TTCE-Wgpu(git repository root)/dxcompiler_build/dxcompiler.dll`

### TTCEWgpuRustCore.g.cs を生成させる

`ttce-wgpu-rust-core` にある rust の プロジェクトを build を行い、 csbindgen から TTCEWgpuRustCore.g.cs を `TTCE-Wgpu(git repository root)/TTCE-Wgpu/TTCEWgpuRustCore.g.cs` に生成させます。

### TexTransTool を指定の場所に置くかリンクを張る

下記のパスに合うようにディレクトリを生成し [TexTransTool](https://github.com/ReinaS-64892/TexTransTool) を git clone してください。

Path: `TTCE-Wgpu(git repository root)/ProjectPackages/TexTransTool`

それか、 UnityProject に TTT を Git で Packages 配下に置いている場合は symlink を `ProjectPackages -> (UnityProjectRoot/)Packages` にリンクを貼ってください。

## PackageDeploy について

これは、ローカルの開発環境向け、そして 上記 TTT を UnityProject に symlink でつなぐ形で配置した場合用の `dotnet run` で使用できるスクリプトです。

Windows環境以外での動作はできません。必要であれば PR をください。

### 何をしているか

諸々の存在をチェックした後、 `TTCE-Wgpu(git repository root)ProjectPackages\TTCE-Wgpu` に配置しに行きます。以前に実行されていて配置されていた場合は先に削除します。

その次に下記の内容を行います。

- Directory `TTCE-Wgpu(git repository root)ProjectPackages\TTCE-Wgpu` を生成
- `TTCE-Wgpu(git repository root)\TTCE-Wgpu\UnityPackageMetaData` の中身を展開
- `TTCE-Wgpu(git repository root)\TTCE-Wgpu\*.cs` でヒットするものを展開
- 出力されたバイナリの中から ttce-wgpu-rust-core の ダイナミックリンク用ライブラリを配置
- `TTCE-Wgpu(git repository root)\dxcompiler_build` にあるダイナミック用ライブラリを配置

ちなみに現在、 DLL をダイナミックリンクできていないので Unity 起動中に行うと失敗します。(初回を除く)
