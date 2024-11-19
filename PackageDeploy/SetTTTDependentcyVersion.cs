

using System.Reflection.Metadata;
using System.Text.Json.Nodes;

public static class SetTTTDependencyVersion
{
    const string TTT_PACKAGE_JASON = @"ProjectPackages\TexTransTool\package.json";
    const string PACKAGE_JASON = @"TTCE-Wgpu\UnityPackageMetaData\package.json";
    public static void Run()
    {
        if (Path.GetFileName(Directory.GetCurrentDirectory()) is "PackageDeploy")
        {
            Directory.SetCurrentDirectory("../");
            Console.WriteLine($"一つ上のディレクトリに移動 ... \"{Directory.GetCurrentDirectory()}\"");
        }


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
}
