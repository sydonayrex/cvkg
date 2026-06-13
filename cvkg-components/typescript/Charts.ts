/**
 * Charts component rendering analytical charts natively on Canvas.
 * 
 * Implements line, bar, pie, and radar visualizations.
 */
export interface ChartDataPoint {
  label: string;
  value: number;
}

export interface ChartProps {
  /** The chart data array. */
  data: ChartDataPoint[];
  /** The type of visual chart (line, bar, pie, radar). */
  type: "line" | "bar" | "pie" | "radar";
}

/**
 * Portable representation of analytical Charts.
 */
export class Charts {
  private data: ChartDataPoint[];
  private type: "line" | "bar" | "pie" | "radar";

  /**
   * Constructs a new Charts instance.
   */
  constructor(props: ChartProps) {
    this.data = props.data;
    this.type = props.type;
  }

  /**
   * Renders the chart visualization onto an HTML Canvas.
   */
  public render(canvas: HTMLCanvasElement): void {
    const ctx = canvas.getContext("2d");
    if (!ctx) return;

    const w = canvas.width;
    const h = canvas.height;
    ctx.clearRect(0, 0, w, h);

    if (this.type === "bar") {
      this.drawBarChart(ctx, w, h);
    } else if (this.type === "line") {
      this.drawLineChart(ctx, w, h);
    } else if (this.type === "pie") {
      this.drawPieChart(ctx, w, h);
    } else if (this.type === "radar") {
      this.drawRadarChart(ctx, w, h);
    }
  }

  /**
   * Draws a bar chart.
   */
  private drawBarChart(ctx: CanvasRenderingContext2D, w: number, h: number): void {
    const padding = 40;
    const chartW = w - padding * 2;
    const chartH = h - padding * 2;

    const maxVal = Math.max(...this.data.map(d => d.value), 1);
    const barW = chartW / this.data.length - 8;

    this.data.forEach((d, i) => {
      const barH = (d.value / maxVal) * chartH;
      const x = padding + i * (barW + 8);
      const y = h - padding - barH;

      ctx.fillStyle = "#0080ff";
      ctx.fillRect(x, y, barW, barH);

      ctx.fillStyle = "#fff";
      ctx.font = "10px sans-serif";
      ctx.fillText(d.label, x, h - padding + 16);
      ctx.fillText(d.value.toString(), x, y - 4);
    });
  }

  /**
   * Draws a line chart.
   */
  private drawLineChart(ctx: CanvasRenderingContext2D, w: number, h: number): void {
    const padding = 40;
    const chartW = w - padding * 2;
    const chartH = h - padding * 2;

    const maxVal = Math.max(...this.data.map(d => d.value), 1);
    const stepX = chartW / (this.data.length - 1 || 1);

    ctx.strokeStyle = "#00ffcc";
    ctx.lineWidth = 2;
    ctx.beginPath();

    this.data.forEach((d, i) => {
      const x = padding + i * stepX;
      const y = h - padding - (d.value / maxVal) * chartH;

      if (i === 0) {
        ctx.moveTo(x, y);
      } else {
        ctx.lineTo(x, y);
      }

      ctx.fillStyle = "#fff";
      ctx.font = "10px sans-serif";
      ctx.fillText(d.label, x - 10, h - padding + 16);
    });

    ctx.stroke();
  }

  /**
   * Draws a pie chart.
   */
  private drawPieChart(ctx: CanvasRenderingContext2D, w: number, h: number): void {
    const center = { x: w / 2, y: h / 2 };
    const radius = Math.min(w, h) / 2 - 20;

    const total = this.data.reduce((sum, d) => sum + d.value, 0) || 1;
    let startAngle = 0;

    const colors = ["#ff5555", "#50fa7b", "#ffb86c", "#bd93f9", "#ff79c6", "#8be9fd"];

    this.data.forEach((d, i) => {
      const sliceAngle = (d.value / total) * Math.PI * 2;
      const endAngle = startAngle + sliceAngle;

      ctx.fillStyle = colors[i % colors.length];
      ctx.beginPath();
      ctx.moveTo(center.x, center.y);
      ctx.arc(center.x, center.y, radius, startAngle, endAngle);
      ctx.closePath();
      ctx.fill();

      startAngle = endAngle;
    });
  }

  /**
   * Draws a radar chart.
   */
  private drawRadarChart(ctx: CanvasRenderingContext2D, w: number, h: number): void {
    const center = { x: w / 2, y: h / 2 };
    const maxRadius = Math.min(w, h) / 2 - 30;

    const maxVal = Math.max(...this.data.map(d => d.value), 1);
    const angleStep = (Math.PI * 2) / this.data.length;

    // Outer boundary polygon
    ctx.strokeStyle = "#444";
    ctx.lineWidth = 1;
    ctx.beginPath();
    this.data.forEach((_, i) => {
      const angle = i * angleStep - Math.PI / 2;
      const x = center.x + maxRadius * Math.cos(angle);
      const y = center.y + maxRadius * Math.sin(angle);
      if (i === 0) ctx.moveTo(x, y);
      else ctx.lineTo(x, y);
    });
    ctx.closePath();
    ctx.stroke();

    // Data polygon
    ctx.strokeStyle = "#ff79c6";
    ctx.fillStyle = "rgba(255, 121, 198, 0.3)";
    ctx.lineWidth = 2;
    ctx.beginPath();
    this.data.forEach((d, i) => {
      const angle = i * angleStep - Math.PI / 2;
      const valRadius = (d.value / maxVal) * maxRadius;
      const x = center.x + valRadius * Math.cos(angle);
      const y = center.y + valRadius * Math.sin(angle);
      if (i === 0) ctx.moveTo(x, y);
      else ctx.lineTo(x, y);
    });
    ctx.closePath();
    ctx.fill();
    ctx.stroke();
  }
}
