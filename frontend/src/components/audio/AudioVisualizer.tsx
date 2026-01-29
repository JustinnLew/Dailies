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
    if (!canvasRef.current) return;
    const source = audio.source;
    const analyser = analyserRef.current;
    const canvasCtx = canvasRef.current.getContext("2d");
    if (!canvasCtx) return;

    source.connect(analyser);
    analyser.fftSize = 128;
    analyser.smoothingTimeConstant = 0.8;
    const bufferLength = analyser.frequencyBinCount;
    const dataArray = new Uint8Array(bufferLength);
    

    function draw() {
      if (!canvasCtx || !canvasRef.current) return;
      const WIDTH = canvasRef.current.width;
      const HEIGHT = canvasRef.current.height;
      
      requestAnimationFrame(draw);
      canvasCtx.clearRect(0, 0, WIDTH, HEIGHT);
      analyser.getByteFrequencyData(dataArray);
      const max = Math.max(...dataArray);
      const min = Math.min(...dataArray);
      const range = max - min;

      const barWidth = (WIDTH / bufferLength) * 1.5;
      let x = 0;
      for (let i = 0; i < dataArray.length; i++) {
        const v = range > 0 ? (dataArray[i] - min) / range : 0;
        const barHeight = v * HEIGHT;
        
        // Gradient from pink to yellow
        const gradient = canvasCtx.createLinearGradient(0, HEIGHT - barHeight, 0, HEIGHT);
        gradient.addColorStop(0, 'oklch(0.85 0.20 90)'); // yellow
        gradient.addColorStop(0.5, 'oklch(0.60 0.30 240)'); // blue
        gradient.addColorStop(1, 'oklch(0.55 0.35 340)'); // pink

        canvasCtx.fillStyle = gradient;
        canvasCtx.shadowBlur = 10;
        canvasCtx.shadowColor = 'oklch(0.55 0.35 340)';
        canvasCtx.fillRect(x, HEIGHT - barHeight, barWidth - 2, barHeight);

        x += barWidth;
      }
    canvasCtx.shadowBlur = 0;
    }
    
    draw();
  }, [audio]);

  return <canvas ref={canvasRef} style={{ width, height }}></canvas>;
}