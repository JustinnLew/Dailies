import { useEffect, useRef } from "react";
import { useAudio } from "./AudioProvider";

export default function AudioVisualizer({
  width,
  height,
}: {
  width: string;
  height: string;
}) {
  const audio = useAudio();
  const analyserRef = useRef<AnalyserNode>(audio.audioCtx.createAnalyser());
  const canvasRef = useRef<HTMLCanvasElement>(null);

  useEffect(() => {
    if (!canvasRef.current || !canvasRef) return;
    const source = audio.source;
    const analyser = analyserRef.current;
    const canvasCtx = canvasRef.current.getContext("2d");
    if (!canvasCtx) return;

    source.connect(analyser);
    analyser.fftSize = 2048;
    const bufferLength = analyser.frequencyBinCount;
    const dataArray = new Uint8Array(bufferLength);

    canvasCtx.clearRect(
      0,
      0,
      canvasRef.current.height,
      canvasRef.current.width,
    );

    function draw() {
      if (!canvasCtx || !canvasRef.current) return;
      const width = canvasRef.current.width;
      const height = canvasRef.current.height;
      requestAnimationFrame(draw);
      analyser.getByteTimeDomainData(dataArray);

      canvasCtx.fillStyle = "rgb(200 200 200)";
      canvasCtx.fillRect(0, 0, width, height);

      const barWidth = (width / bufferLength) * 2.5;
      let barHeight;
      let x = 0;
      for (let i = 0; i < bufferLength; i++) {
        barHeight = dataArray[i] / 2;

        canvasCtx.fillStyle = `rgb(${barHeight + 100} 50 50)`;
        canvasCtx.fillRect(x, height - barHeight / 2, barWidth, barHeight);

        x += barWidth + 1;
      }
    }
    draw();
  }, [audio]);

  return <canvas ref={canvasRef} style={{ width, height }}></canvas>;
}
