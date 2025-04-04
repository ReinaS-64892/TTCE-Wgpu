using net.rs64.TexTransCore;
using net.rs64.TexTransCore.MultiLayerImageCanvas;
using net.rs64.TexTransCoreEngineForWgpu;
using net.rs64.TexTransCoreEngineForWgpu.Tests;
using SixLabors.ImageSharp;
using SixLabors.ImageSharp.PixelFormats;


Console.WriteLine("Test Run");
Console.WriteLine("debug - CurrentDirectory:" + Directory.GetCurrentDirectory());


using var engineTest = new TestTTCEWgpuEngine(TexTransCoreTextureFormat.Byte, true, TTCEWgpuDevice.RequestDevicePreference.IntegratedGPUOrCPU);
var ctx = engineTest.GetCtx();

// {
// using var rt = ctx.CreateRenderTexture(256, 256, TexTransCoreTextureChannel.R);

// ctx.FillR(rt, 1f);

// Span<byte> outByte = stackalloc byte[rt.Width * rt.Hight * 4];
// ctx.DownloadTexture(outByte, TexTransCoreTextureFormat.Byte, rt);

// foreach (var v in outByte)
// {
//     Console.Write(v);
// }
// }

// {
//     var rt = ctx.CreateRenderTexture(2048, 2048);

//     var imageBuf = new byte[rt.Width * rt.Hight * EnginUtil.GetPixelParByte(TexTransCoreTextureFormat.Byte, rt.ContainsChannel)];
//     using (var img = Image.Load<Rgba32>(@"D:\Rs\TTCE-Wgpu\TTCE-Wgpu.Tests\TestData\0-Hair.png"))
//         img.CopyPixelDataTo(imageBuf);

//     ctx.UploadTexture<byte>(rt, imageBuf.AsSpan(), TexTransCoreTextureFormat.Byte);

// var selectiveColor = new SelectiveColorAdjustment(new(), new(), new(), new(), new(), new(), new(), new(), new(0,0,0,-0.3f), false);
//     selectiveColor.GrabBlending(ctx, rt);

//     ctx.DownloadTexture<byte>(imageBuf, TexTransCoreTextureFormat.Byte, rt);
//     using (var outImage = Image<Rgba32>.LoadPixelData<Rgba32>(imageBuf.AsSpan(), 2048, 2048))
//     using (var fsStream = File.Open(@"D:\Rs\TTCE-Wgpu\TTCE-Wgpu.Tests\TestData\0-Result.png", FileMode.OpenOrCreate))
//         outImage.SaveAsPng(fsStream);
// }
