using System.Collections;
using net.rs64.TexTransCore;
using Xunit;

namespace net.rs64.TexTransCoreEngineForWgpu.Tests;

public class TestTTCEWgpuEngine : IDisposable
{
    public const string ShaderFindingPath = "ProjectPackages/TexTransTool";
    public TTCEWgpuDevice Device;
    private readonly ShaderFinder.ShaderDictionary _shaderDict;
    private readonly TTCEWgpuRCDebugPrintToConsole? _debugLogHandler;

    public TestTTCEWgpuEngine(TexTransCoreTextureFormat defaultFormat = TexTransCoreTextureFormat.Byte)
    {
        Device = new TTCEWgpuDevice();
        Device.SetDefaultTextureFormat(defaultFormat);
        _shaderDict = ShaderFinder.RegisterShaders(Device, ShaderFinder.GetAllShaderPathWithCurrentDirectory(), ShaderFinder.CurrentDirectoryFind);
        // _debugLogHandler = new TTCEWgpuRCDebugPrintToConsole();
        _debugLogHandler = null;
    }
    public TTCEWgpuContextWithShaderDictionary GetCtx()
    {
        var ctx = Device.GetContext<TTCEWgpuContextWithShaderDictionary>();
        ctx.ShaderDictionary = _shaderDict;
        return ctx;
    }
    public void Dispose()
    {
        Device.Dispose();
        _debugLogHandler?.Dispose();
    }
}

public class TestTTCEWgpuEngineData : IEnumerable<object[]>, IDisposable
{
    List<TestTTCEWgpuEngine> _testData;
    public TestTTCEWgpuEngineData()
    {
        _testData =
        [
            new TestTTCEWgpuEngine(TexTransCoreTextureFormat.Byte),
            new TestTTCEWgpuEngine(TexTransCoreTextureFormat.UShort),
            new TestTTCEWgpuEngine(TexTransCoreTextureFormat.Half),
            new TestTTCEWgpuEngine(TexTransCoreTextureFormat.Float),
        ];
    }

    public void Dispose()
    {
        foreach (var e in _testData) { e.Dispose(); }
    }
    public IEnumerator<object[]> GetEnum() => _testData.Select<TestTTCEWgpuEngine, object[]>(e => [e]).GetEnumerator();

    public IEnumerator<object[]> GetEnumerator() => GetEnum();
    IEnumerator IEnumerable.GetEnumerator() => GetEnum();

}
