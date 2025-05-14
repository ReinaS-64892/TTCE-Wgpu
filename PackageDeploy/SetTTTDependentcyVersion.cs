

using System.Reflection.Metadata;
using System.Text.Json.Nodes;

public static class SetTTTDependencyVersion
{
    const string TTC_PACKAGE_JASON = @"ProjectPackages/TexTransCore/package.json";
    const string PACKAGE_JASON = @"TTCE-Wgpu/UnityPackageMetaData/package.json";
    const string TTT_EDITOR_ASMDEF = @"ProjectPackages/TexTransTool/Editor/net.rs64.tex-trans-tool.editor.asmdef";
    const string TTT_RUNTIME_ASMDEF = @"ProjectPackages/TexTransTool/Runtime/net.rs64.tex-trans-tool.runtime.asmdef";
    const string TTT_NDMF_ASMDEF = @"ProjectPackages/TexTransTool/Editor/NDMF/net.rs64.tex-trans-tool.ndmf.asmdef";

    public static void WriteTTCDependVersion()
    {
        var packageJson = JsonNode.Parse(File.ReadAllText(PACKAGE_JASON));
        var ttcPackageJson = JsonNode.Parse(File.ReadAllText(TTC_PACKAGE_JASON));

        if (packageJson is null || ttcPackageJson is null)
        {
            Console.WriteLine($"Json parse failed!");
            throw new NullReferenceException();
        }


        var ttcId = "net.rs64.tex-trans-core";
        var dependencies = packageJson["dependencies"];
        if (dependencies is null) { Console.WriteLine($"ttt dependency not found!"); throw new NullReferenceException(); }
        dependencies[ttcId] = ttcPackageJson["version"]?.GetValue<string>();
        var vpmDependencies = packageJson["vpmDependencies"];
        if (vpmDependencies is null) { Console.WriteLine($"ttt dependency not found!"); throw new NullReferenceException(); }
        vpmDependencies[ttcId] = "^" + ttcPackageJson["version"]?.GetValue<string>();

        var outOpt = new System.Text.Json.JsonSerializerOptions(System.Text.Json.JsonSerializerDefaults.General);
        outOpt.WriteIndented = true;
        File.WriteAllText(PACKAGE_JASON, packageJson.ToJsonString(outOpt) + "\n");
    }
    public static void WriteTTTDependentTTCEWgpuVersion()
    {
        WriteTTTDependentTTCEWgpuVersionForAsmDef(TTT_EDITOR_ASMDEF);
        WriteTTTDependentTTCEWgpuVersionForAsmDef(TTT_NDMF_ASMDEF);
        WriteTTTDependentTTCEWgpuVersionForAsmDef(TTT_RUNTIME_ASMDEF);
    }

    private static void WriteTTTDependentTTCEWgpuVersionForAsmDef(string targetAsmDefPath)
    {
        var asmDef = JsonNode.Parse(File.ReadAllText(targetAsmDefPath));
        var packageJson = JsonNode.Parse(File.ReadAllText(PACKAGE_JASON));

        if (packageJson is null || asmDef is null)
        {
            Console.WriteLine($"Json parse failed!");
            throw new NullReferenceException();
        }

        var ttceWgpuId = "net.rs64.ttce-wgpu";
        var ttceWgpuVersion = packageJson["version"]?.GetValue<string>();

        var defines = asmDef["versionDefines"];
        var ttceWgpuDef = defines?.AsArray().First(i => i?["name"]?.GetValue<string>() == ttceWgpuId);
        if (ttceWgpuDef is null) { Console.WriteLine($" TTCE-Wgpu Define not found!"); throw new NullReferenceException(); }
        ttceWgpuDef["expression"] = $"[{ttceWgpuVersion}]";

        var outOpt = new System.Text.Json.JsonSerializerOptions(System.Text.Json.JsonSerializerDefaults.General);
        outOpt.WriteIndented = true;
        File.WriteAllText(targetAsmDefPath, asmDef.ToJsonString(outOpt) + "\n");
    }
}
