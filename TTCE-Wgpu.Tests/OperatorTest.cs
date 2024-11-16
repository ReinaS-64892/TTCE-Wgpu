using net.rs64.TexTransCore;
using Xunit;

namespace net.rs64.TexTransCoreEngineForWgpu.Tests;
public class OperatorTest
{

    [Theory]
    [ClassData(typeof(TestTTCEWgpuEngineData))]
    public void CopyRenderTexture(TestTTCEWgpuEngine device)
    {
        using var ctx = device.GetCtx();

        var sourceDataSpan = new byte[64 * 64 * 4].AsSpan();
        new Random(64).NextBytes(sourceDataSpan);
        var downloadedDataSpan = new byte[64 * 64 * 4].AsSpan();

        using var rtFrom = ctx.CreateRenderTexture(64, 64);
        using var rtTo = ctx.CreateRenderTexture(64, 64);

        ctx.UploadTexture<byte>(rtFrom, sourceDataSpan, TexTransCoreTextureFormat.Byte);

        ctx.CopyRenderTexture(rtTo, rtFrom);

        ctx.DownloadTexture(downloadedDataSpan, TexTransCoreTextureFormat.Byte, rtTo);

        Assert.Equal(sourceDataSpan.Length, downloadedDataSpan.Length);

        for (var i = 0; sourceDataSpan.Length > i; i += 1)
        {
            Assert.Equal(sourceDataSpan[i], downloadedDataSpan[i]);
        }
    }

    [Theory]
    [ClassData(typeof(TestTTCEWgpuEngineData))]
    public void NotEqualCopyRenderTexture(TestTTCEWgpuEngine device)
    {
        using var ctx = device.GetCtx();

        using var rtFrom = ctx.CreateRenderTexture(128, 64);
        using var rtTo = ctx.CreateRenderTexture(64, 64);

        Assert.Throws<ArgumentException>(() => { ctx.CopyRenderTexture(rtTo, rtFrom); });
    }
    [Theory]
    [ClassData(typeof(TestTTCEWgpuEngineData))]
    public void SizeEqualTest(TestTTCEWgpuEngine device)
    {
        using var ctx = device.GetCtx();

        using var rt = ctx.CreateRenderTexture(64, 64);
        using var rt1 = ctx.CreateRenderTexture(64, 64);
        using var rt2 = ctx.CreateRenderTexture(128, 128);
        using var rt4 = ctx.CreateRenderTexture(4096, 4096);

        Assert.True(rt.EqualSize(rt1));
        Assert.False(rt.EqualSize(rt2));
        Assert.False(rt.EqualSize(rt4));
    }
    [Theory]
    [ClassData(typeof(TestTTCEWgpuEngineData))]
    public void SwizzlingTest(TestTTCEWgpuEngine device)
    {
        using var ctx = device.GetCtx();

        var color = new Color(255 / 255f, 100 / 255f, 128 / 255f, 255 / 255f);
        var data = new Color[64 * 64];
        var dataSpan = data.AsSpan();

        using var rt = ctx.CreateRenderTexture(64, 64);
        ctx.ColorFill(rt, color);
        ctx.Swizzling(rt, RenderTextureOperator.SwizzlingChannel.A, RenderTextureOperator.SwizzlingChannel.B, RenderTextureOperator.SwizzlingChannel.G, RenderTextureOperator.SwizzlingChannel.R);

        ctx.DownloadTexture(dataSpan, TexTransCoreTextureFormat.Float, rt);

        for (var i = 0; dataSpan.Length > i; i += 1)
        {
            Assert.Equal(color.R, dataSpan[i].A, 1 / 255f);
            Assert.Equal(color.G, dataSpan[i].B, 1 / 255f);
            Assert.Equal(color.B, dataSpan[i].G, 1 / 255f);
            Assert.Equal(color.A, dataSpan[i].R, 1 / 255f);
        }
    }


    [Theory]
    [ClassData(typeof(TestTTCEWgpuEngineData))]
    public void BilinearRescalingTest(TestTTCEWgpuEngine device)
    {
        using var ctx = device.GetCtx();

        var color = new Color(128 / 255f, 64 / 255f, 64 / 255f, 255 / 255f);
        var data = new Color[128 * 128];
        var dataSpan = data.AsSpan();

        using var rt = ctx.CreateRenderTexture(64, 64);
        using var rt2 = ctx.CreateRenderTexture(128, 128);
        ctx.ColorFill(rt, color);
        ctx.BilinearReScaling(rt2, rt);

        ctx.DownloadTexture(dataSpan, TexTransCoreTextureFormat.Float, rt2);

        for (var i = 0; dataSpan.Length > i; i += 1)
        {
            Assert.Equal(color.R, dataSpan[i].R, 1 / 255f);
            Assert.Equal(color.G, dataSpan[i].G, 1 / 255f);
            Assert.Equal(color.B, dataSpan[i].B, 1 / 255f);
            Assert.Equal(color.A, dataSpan[i].A, 1 / 255f);
        }
    }
}
