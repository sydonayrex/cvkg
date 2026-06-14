import math

def get_triangle(cx, cy, r, angle_offset=0):
    pts = []
    for i in range(3):
        a = math.radians(angle_offset + 90 + i * 120)
        pts.append((cx + r * math.cos(a), cy - r * math.sin(a)))
    return f"M{pts[0][0]:.1f},{pts[0][1]:.1f} L{pts[1][0]:.1f},{pts[1][1]:.1f} L{pts[2][0]:.1f},{pts[2][1]:.1f} Z"

t1 = get_triangle(50, 40, 25, 0)
t2 = get_triangle(40, 60, 25, 0)
t3 = get_triangle(60, 60, 25, 0)

svg = f"""<svg viewBox="0 0 100 100" xmlns="http://www.w3.org/2000/svg">
  <g transform="translate(0, 0)">
    <path id="t1" fill="none" stroke="#FF4000" stroke-width="4" d="{t1}" />
    <path id="t2" fill="none" stroke="#FF8000" stroke-width="4" d="{t2}" />
    <path id="t3" fill="none" stroke="#FFC000" stroke-width="4" d="{t3}" />
    <animate attributeName="stroke-dashoffset" from="1" to="0" dur="2s" repeatCount="indefinite" />
  </g>
</svg>"""

print(svg)
