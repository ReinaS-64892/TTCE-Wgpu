using System.Collections;
using net.rs64.TexTransCore;
using Xunit;

namespace net.rs64.TexTransCoreEngineForWgpu.Tests;

public class TestTTCEWgpuEngine : IDisposable
{
    public TTCEWgpuDeviceWidthShaderDictionary Device;
    private readonly TTCEWgpuRCDebugPrintToConsole? _debugLogHandler;

    public TestTTCEWgpuEngine(TexTransCoreTextureFormat defaultFormat = TexTransCoreTextureFormat.Byte, bool showDebugLog = false, TTCEWgpuDevice.RequestDevicePreference requestDevice = TTCEWgpuDevice.RequestDevicePreference.Auto)
    {
        _debugLogHandler = showDebugLog ? new TTCEWgpuRCDebugPrintToConsole() : null;
        Device = new TTCEWgpuDeviceWidthShaderDictionary(requestDevice, defaultFormat);
    }
    public TTCEWgpuContextWithShaderDictionary GetCtx()
    {
        return Device.GetTTCEWgpuContext();
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
