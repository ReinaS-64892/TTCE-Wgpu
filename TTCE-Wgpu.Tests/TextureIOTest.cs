using net.rs64.TexTransCore;
using Xunit;

namespace net.rs64.TexTransCoreEngineForWgpu.Tests;
public class TextureIOTest
{
    [Theory]
    [ClassData(typeof(TestTTCEWgpuEngineData))]
    public void DownloadTest(TestTTCEWgpuEngine device)
    {
        using var ctx = device.GetCtx();

        var color = new Color(255 / 255f, 100 / 255f, 128 / 255f, 255 / 255f);
        var data = new Color[64 * 64];
        var dataSpan = data.AsSpan();

        using var rt = ctx.CreateRenderTexture(64, 64);
        ctx.ColorFill(rt, color);

        ctx.DownloadTexture(dataSpan, TexTransCoreTextureFormat.Float, rt);

        for (var i = 0; dataSpan.Length > i; i += 1)
        {
            Assert.Equal(color.R, dataSpan[i].R, 1 / 255f);
            Assert.Equal(color.G, dataSpan[i].G, 1 / 255f);
            Assert.Equal(color.B, dataSpan[i].B, 1 / 255f);
            Assert.Equal(color.A, dataSpan[i].A, 1 / 255f);
        }
    }

    [Theory]
    [ClassData(typeof(TestTTCEWgpuEngineData))]
    public void UploadTest(TestTTCEWgpuEngine device)
    {
        using var ctx = device.GetCtx();

        var sourceDataSpan = new byte[64 * 64 * 4].AsSpan();
        new Random(64).NextBytes(sourceDataSpan);
        var downloadedDataSpan = new byte[64 * 64 * 4].AsSpan();

        using var rt = ctx.CreateRenderTexture(64, 64);
        ctx.UploadTexture<byte>(rt, sourceDataSpan, TexTransCoreTextureFormat.Byte);
        ctx.DownloadTexture(downloadedDataSpan, TexTransCoreTextureFormat.Byte, rt);

        Assert.Equal(sourceDataSpan.Length, downloadedDataSpan.Length);

        for (var i = 0; sourceDataSpan.Length > i; i += 1)
        {
            Assert.Equal(sourceDataSpan[i], downloadedDataSpan[i]);
        }
    }
}
