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
/** In split view the whole pane under the pointer is a drop target (not only the edges): the pane is found with the
 *  same geometry as the split itself. @param {{left:number,top:number,width:number,height:number}} bounds @param {number} ratio @param {number} x @param {number} y */
export function splitPaneDropSide(bounds, ratio, x, y) {
  if (bounds.width < 208 || bounds.height < 100 || x < bounds.left || x > bounds.left + bounds.width || y < bounds.top || y > bounds.top + bounds.height) return null;
  const available = Math.max(200, bounds.width - 8);
  const left = Math.max(100, Math.min(available - 100, available * Math.max(.2, Math.min(.8, ratio))));
  return x < bounds.left + left + 4 ? 'left' : 'right';
}
/** @param {number} startX @param {number} startY @param {number} x @param {number} y */
export const crossedDragThreshold = (startX, startY, x, y) => Math.hypot(x - startX, y - startY) >= 6;
