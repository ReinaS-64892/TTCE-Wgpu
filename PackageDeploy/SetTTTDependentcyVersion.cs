

using System.Reflection.Metadata;
using System.Text.Json.Nodes;

public static class SetTTTDependencyVersion
{
    const string TTT_PACKAGE_JASON = @"ProjectPackages\TexTransTool\package.json";
    const string PACKAGE_JASON = @"TTCE-Wgpu\UnityPackageMetaData\package.json";
    const string TTT_EDITOR_ASMDEF = @"ProjectPackages\TexTransTool\Editor\net.rs64.tex-trans-tool.editor.asmdef";
    public static void WriteTTTDependVersion()
    {
        var packageJson = JsonNode.Parse(File.ReadAllText(PACKAGE_JASON));
        var tttPackageJson = JsonNode.Parse(File.ReadAllText(TTT_PACKAGE_JASON));

        if (packageJson is null || tttPackageJson is null)
        {
            Console.WriteLine($"Json parse failed!");
            throw new NullReferenceException();
        }


        var tttId = "net.rs64.tex-trans-tool";
        var dependencies = packageJson["dependencies"];
        if (dependencies is null) { Console.WriteLine($"ttt dependency not found!"); throw new NullReferenceException(); }
        dependencies[tttId] = tttPackageJson["version"]?.GetValue<string>();

        var outOpt = new System.Text.Json.JsonSerializerOptions(System.Text.Json.JsonSerializerDefaults.General);
        outOpt.WriteIndented = true;
        File.WriteAllText(PACKAGE_JASON, packageJson.ToJsonString(outOpt) + "\n");
    }
    public static void WriteTTTDependentTTCEWgpuVersion()
    {
        var asmDef = JsonNode.Parse(File.ReadAllText(TTT_EDITOR_ASMDEF));
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
        File.WriteAllText(TTT_EDITOR_ASMDEF, asmDef.ToJsonString(outOpt) + "\n");
    }
}
