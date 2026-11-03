/** @param {{left:number,top:number,width:number,height:number}} bounds @param {number} x @param {number} y */
export function splitDropSide(bounds, x, y) {
  if (bounds.width < 208 || bounds.height < 100 || x < bounds.left || x > bounds.left + bounds.width || y < bounds.top || y > bounds.top + bounds.height) return null;
  if (x <= bounds.left + 32) return 'left';
  if (x >= bounds.left + bounds.width - 32) return 'right';
  return null;
}
/** @param {number} startX @param {number} startY @param {number} x @param {number} y */
export const crossedDragThreshold = (startX, startY, x, y) => Math.hypot(x - startX, y - startY) >= 6;
