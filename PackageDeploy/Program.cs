using System;
using System.Diagnostics;

internal class Program
{
    const string PROJECT_PACKAGES = "ProjectPackages";
    const string TEXTRANSTOOL = "TexTransTool";
    const string TEXTRANSCORE = "TexTransCore";
    const string TTCE_WGPU = "TTCE-Wgpu";

    const string TTCE_WGPU_RUST_CORE_DLL = "ttce_wgpu_rust_core.dll";
    const string TTCE_WGPU_RUST_CORE_SO = "libttce_wgpu_rust_core.so";

    const string TTCE_WGPU_DLL = "net.rs64.ttce-wgpu.dll";

    const string DIRECX_SHADER_COMPILER_DLL = "dxcompiler.dll";
    const string DIRECX_SHADER_COMPILER_SO = "libdxcompiler.so";

    const string DIRECX_SHADER_COMPILER_BUILD_DIR = "dxcompiler_build";
    const string UNITY_PACKAGE_META_DATA = "UnityPackageMetaData";
    /// <summary>
    /// ローカル環境向けのパッケージデプロイ用のスクリプトです。
    /// Windows環境以外向けには作られておらず、どうなるかわからないですが...ほしい人は contribution してね！
    /// </summary>
    static void Main(string[] args)
    {
        Console.WriteLine("Nya!");
        Console.WriteLine("ProjectPackages に対して TTCE-Wgpu の Package を作成しに行くよ～！");


        if (Path.GetFileName(Directory.GetCurrentDirectory()) is not "PackageDeploy") { Console.WriteLine($"CurrentDirectory がおかしいよ！！ {Directory.GetCurrentDirectory()}"); }


        Directory.SetCurrentDirectory("../");
        Console.WriteLine($"一つ上のディレクトリに移動 ... \"{Directory.GetCurrentDirectory()}\"");


        Console.Write("Packages への Symlink の存在確認 ... ");
        if (Directory.Exists(PROJECT_PACKAGES)) Console.WriteLine("Ok!");
        else { Console.WriteLine("Err!"); return; }


        Console.Write("Packages 内の TexTransTool の存在確認 ... ");
        if (Directory.Exists(Path.Combine(PROJECT_PACKAGES, TEXTRANSTOOL))) Console.WriteLine("Ok!");
        else { Console.WriteLine("Err!"); return; }

        Console.Write("Packages 内の TexTransCore の存在確認 ... ");
        if (Directory.Exists(Path.Combine(PROJECT_PACKAGES, TEXTRANSCORE))) Console.WriteLine("Ok!");
        else { Console.WriteLine("Err!"); return; }

        Console.Write("dotnet の存在確認 ... ");
        using (var dotnetProc = Process.Start("dotnet", "--version"))
        {
            dotnetProc.WaitForExit();
            if (dotnetProc.ExitCode != 0) { Console.WriteLine("Err!"); return; }
        }


        Console.Write("cargo の存在確認 ... ");
        using (var dotnetProc = Process.Start("cargo", "--version"))
        {
            dotnetProc.WaitForExit();
            if (dotnetProc.ExitCode != 0) { Console.WriteLine("Err!"); return; }
        }

        Console.WriteLine("");
        Console.WriteLine("Write TTT version");
        SetTTTDependencyVersion.WriteTTTDependVersion();
        Console.WriteLine("Write TTCE-Wgpu version");
        SetTTTDependencyVersion.WriteTTTDependentTTCEWgpuVersion();
        Console.WriteLine("");


        Console.WriteLine("TTCE-Wgpu build!");

        Console.WriteLine("");
        Console.WriteLine("BEGIN DOTNET");
        var buildProcessInfo = new ProcessStartInfo()
        {
            WorkingDirectory = TTCE_WGPU,
            FileName = "dotnet",
            Arguments = "build",
        };

        using (var dotnetProc = Process.Start(buildProcessInfo))
        {
            if (dotnetProc is null) { Console.WriteLine("Build fail!"); return; }
            dotnetProc.WaitForExit();
            if (dotnetProc.ExitCode != 0) { Console.WriteLine("Err!"); return; }
        }
        Console.WriteLine("END DOTNET");
        Console.WriteLine("");

        string rustCoreDLLPath;
        string directXShaderCompilerDLLPath;
        if (OperatingSystem.IsWindows())
        {
            Console.Write("RustCore DLL チェック ... ");
            rustCoreDLLPath = Path.Combine(TTCE_WGPU, "bin", TTCE_WGPU_RUST_CORE_DLL);
            if (File.Exists(rustCoreDLLPath)) Console.WriteLine("Ok!");
            else { Console.WriteLine("Err!"); return; }

            Console.Write("DirectXShaderCompiler DLL チェック ... ");
             directXShaderCompilerDLLPath = Path.Combine(DIRECX_SHADER_COMPILER_BUILD_DIR, DIRECX_SHADER_COMPILER_DLL);
            if (File.Exists(directXShaderCompilerDLLPath)) Console.WriteLine("Ok!");
            else { Console.WriteLine("Err!"); return; }
        }
        else if (OperatingSystem.IsLinux())
        {
            Console.Write("RustCore SO チェック ... ");
             rustCoreDLLPath = Path.Combine(TTCE_WGPU, "bin", TTCE_WGPU_RUST_CORE_SO);
            if (File.Exists(rustCoreDLLPath)) Console.WriteLine("Ok!");
            else { Console.WriteLine("Err!"); return; }

            Console.Write("DirectXShaderCompiler SO チェック ... ");
             directXShaderCompilerDLLPath = Path.Combine(DIRECX_SHADER_COMPILER_BUILD_DIR, DIRECX_SHADER_COMPILER_SO);
            if (File.Exists(directXShaderCompilerDLLPath)) Console.WriteLine("Ok!");
            else { Console.WriteLine("Err!"); return; }
        }
        else
        {
            Console.WriteLine("未知の OS です！！！");
            return;
        }



        Console.Write("UnityPackageMetaData の存在確認 ... ");
        if (Directory.Exists(Path.Combine(TTCE_WGPU, UNITY_PACKAGE_META_DATA))) Console.WriteLine("Ok!");
        else { Console.WriteLine("Err!"); return; }


        Console.Write("TTCE-Wgpu がすでに出力されているか ... ");
        var ttcePackagePath = Path.Combine(PROJECT_PACKAGES, TTCE_WGPU);
        var alreadyTTCE = Directory.Exists(ttcePackagePath);
        if (alreadyTTCE) Console.WriteLine("Some!");
        else Console.WriteLine("None!");


        if (alreadyTTCE)
        {
            Console.WriteLine("Remove before TTCE-Wgpu package");
            Directory.Delete(ttcePackagePath, true);
        }
        else { Console.WriteLine("Skip remove TTCE-Wgpu package"); }


        Console.WriteLine("Create TTCE-Wgpu package directory");
        Directory.CreateDirectory(ttcePackagePath);


        Console.WriteLine("Copy UnityPackage meta data");
        foreach (var filePath in Directory.GetFiles(Path.Combine(TTCE_WGPU, UNITY_PACKAGE_META_DATA)))
        {
            Console.Write($" {Path.GetFileName(filePath)} ");
            File.Copy(filePath, Path.Combine(ttcePackagePath, Path.GetFileName(filePath)));
        }


        Console.WriteLine("");
        Console.WriteLine("Copy Scripts");
        foreach (var filePath in Directory.GetFiles(Path.Combine(TTCE_WGPU)))
        {
            if (Path.GetExtension(filePath) != ".cs") { continue; }

            Console.Write($" {Path.GetFileName(filePath)} ");
            File.Copy(filePath, Path.Combine(ttcePackagePath, Path.GetFileName(filePath)));
        }


        Console.WriteLine("");
        Console.WriteLine("Copy DLLs");

        Console.Write($" {Path.GetFileName(rustCoreDLLPath)} ");
        File.Copy(rustCoreDLLPath, Path.Combine(ttcePackagePath, Path.GetFileName(rustCoreDLLPath)));

        Console.Write($" {Path.GetFileName(directXShaderCompilerDLLPath)} ");
        File.Copy(directXShaderCompilerDLLPath, Path.Combine(ttcePackagePath, Path.GetFileName(directXShaderCompilerDLLPath)));


        Console.WriteLine("");
        Console.WriteLine("Exit!");
    }
}

