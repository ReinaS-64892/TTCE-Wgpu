using net.rs64.TexTransCore;
using net.rs64.TexTransCoreEngineForWgpu;
using SixLabors.ImageSharp;
using SixLabors.ImageSharp.Formats;
using SixLabors.ImageSharp.PixelFormats;


public class TestClass
{
    public static void Func()
    {
        Console.WriteLine("にゃ！");

        using (var ttceDevice = new TTCEWgpuDevice())
        {
            var str = @"F:\unityproject\Lime - ReinaSEdit\Packages\TexTransTool\TexTransCore\ShaderAssets\GrabBlend\LevelAdjustment.ttcomp";
            var id = ttceDevice.RegisterComputeShaderFromHLSL(str);
            Console.WriteLine("the registered id is " + id);

            using (var ttceCtx = ttceDevice.GetContext<TTCEWgpu>())
            using (var rt = ttceCtx.GetRenderTexture(2048, 2048))
            {
                var imageBuf = new byte[2048 * 2048 * EnginUtil.GetPixelParByte(TexTransCoreTextureFormat.Byte, TexTransCoreTextureChannel.RGBA)];
                using (var img = Image.Load<Rgba32>(@"D:\Rs\TTCE-Wgpu\TTCE-Wgpu.Test\TestData\0-Hair.png"))
                    img.CopyPixelDataTo(imageBuf);

                Console.WriteLine("Upload!");
                ttceCtx.UploadTexture<byte>(rt, imageBuf.AsSpan(), TexTransCoreTextureFormat.Byte);

                using (var compute_holder = ttceCtx.GetTTComputeHandler(id))
                {
                    var texId = compute_holder.NameToID("Tex");
                    var cbID = compute_holder.NameToID("gv");
                    compute_holder.SetRenderTexture(texId, rt);

                    var input_floor = 0f;
                    var input_ceiling = 1f;
                    var gamma = 0.3f;
                    var output_floor = 0f;
                    var output_ceiling = 0.9f;
                    var r = 1f;
                    var g = 1f;
                    var b = 1f;

                    Span<byte> data = stackalloc byte[32];

                    BitConverter.TryWriteBytes(data.Slice(0, 4), input_floor);
                    BitConverter.TryWriteBytes(data.Slice(4, 4), input_ceiling);
                    BitConverter.TryWriteBytes(data.Slice(8, 4), gamma);
                    BitConverter.TryWriteBytes(data.Slice(12, 4), output_floor);
                    BitConverter.TryWriteBytes(data.Slice(16, 4), output_ceiling);
                    BitConverter.TryWriteBytes(data.Slice(20, 4), r);
                    BitConverter.TryWriteBytes(data.Slice(24, 4), g);
                    BitConverter.TryWriteBytes(data.Slice(28, 4), b);

                    compute_holder.UploadConstantsBuffer<byte>(cbID, data);

                    var wg_size = compute_holder.GetWorkGroupSize();
                    compute_holder.Dispatch(Math.Max(rt.GetWidth() / wg_size.x, 1), Math.Max(rt.GetHeight() / wg_size.y, 1), 1);
                }

                Console.WriteLine("Download!");
                ttceCtx.DownloadTexture<byte>(imageBuf, TexTransCoreTextureFormat.Byte, rt);
                using (var outImage = Image<Rgba32>.LoadPixelData<Rgba32>(imageBuf.AsSpan(), 2048, 2048))
                using (var fsStream = File.Open(@"D:\Rs\TTCE-Wgpu\TTCE-Wgpu.Test\TestData\0-Result.png", FileMode.OpenOrCreate))
                    outImage.SaveAsPng(fsStream);
            }
        }
    }
}
