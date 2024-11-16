using net.rs64.TexTransCore;
using Xunit;

namespace net.rs64.TexTransCoreEngineForWgpu.Tests;
public class DisposedTest
{
    [Theory]
    [ClassData(typeof(TestTTCEWgpuEngineData))]
    public void ContextTest(TestTTCEWgpuEngine device)
    {
        using var ctx = device.GetCtx();


        ctx.Dispose();
        Assert.Throws<ObjectDisposedException>(() => { _ = ctx.GetRenderTexture(64, 64); });
    }

    [Theory]
    [ClassData(typeof(TestTTCEWgpuEngineData))]
    public void RenderTextureTest(TestTTCEWgpuEngine device)
    {
        using var ctx = device.GetCtx();

        using var rt = ctx.CreateRenderTexture(64, 64);

        rt.Dispose();
        Assert.Throws<ObjectDisposedException>(() => { _ = rt.Width; });
    }

    [Theory]
    [ClassData(typeof(TestTTCEWgpuEngineData))]
    public void ComputeHandlerTest(TestTTCEWgpuEngine device)
    {
        using var ctx = device.GetCtx();

        using var ch = ctx.GetComputeHandler(ctx.StandardComputeKey.AlphaCopy);

        ch.Dispose();
        Assert.Throws<ObjectDisposedException>(() => { ch.UploadConstantsBuffer(0, [1]); });
    }
}
