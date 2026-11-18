/** How far from the left or right edge of the page area a tab may be dropped to open a split: 15% of the width, between 96 and 240 pixels. @param {number} width */
export const splitEdgeWidth = width => Math.min(240, Math.max(96, width * 0.15));
/** @param {{left:number,top:number,width:number,height:number}} bounds @param {number} x @param {number} y */
export function splitDropSide(bounds, x, y) {
  if (bounds.width < 208 || bounds.height < 100 || x < bounds.left || x > bounds.left + bounds.width || y < bounds.top || y > bounds.top + bounds.height) return null;
  const edge = splitEdgeWidth(bounds.width);
  if (x <= bounds.left + edge) return 'left';
  if (x >= bounds.left + bounds.width - edge) return 'right';
  return null;
}
/** @param {number} startX @param {number} startY @param {number} x @param {number} y */
export const crossedDragThreshold = (startX, startY, x, y) => Math.hypot(x - startX, y - startY) >= 6;
