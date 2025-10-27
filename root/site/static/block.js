import { game } from "./main.js";

const BLOCK_GRP = document.getElementById('block-grp').children;

const randomDX = () => Math.random() + 1;
const randomDY = () => Math.random() + 1;

export const block = {
    grp: BLOCK_GRP,
    x: 0,
    y: 0,
    size: 150,
    dx: randomDX(),
    dy: randomDY(),
}

export async function initBlock(){
    block.x = Math.random() * (game.width - 300) + 150;
    block.y = Math.random() * (game.height - 400) + 200;
    setBlockPosition();
}

export async function moveBlock() {
    block.x += block.dx;
    block.y += block.dy;

    if (block.x > game.width-block.size || block.x < 0) {
        block.dx = (block.dx > 0) ? -randomDX() : randomDX() ;
        block.dy = (block.dy > 0) ? randomDY() : -randomDY() ;
        block.x = (block.x < 0) ? 0 : game.width-block.size;
    };
    if (block.y > game.height-block.size-52 || block.y < 52) {
        block.dx = (block.dx > 0) ? randomDX() : -randomDX() ;
        block.dy = (block.dy > 0) ? -randomDY() : randomDY() ;
        block.y = (block.y < 52) ? 52 : game.height-block.size-52;
    }
    setBlockPosition();
};

function setBlockPosition(){
    block.grp[0].setAttribute('x', block.x);
    block.grp[0].setAttribute('y', block.y);
    block.grp[1].setAttribute('cx', block.x+10);
    block.grp[1].setAttribute('cy', block.y+10);
    block.grp[2].setAttribute('cx', block.x+140);
    block.grp[2].setAttribute('cy', block.y+10);
    block.grp[3].setAttribute('cx', block.x+10);
    block.grp[3].setAttribute('cy', block.y+140);
    block.grp[4].setAttribute('cx', block.x+140);
    block.grp[4].setAttribute('cy', block.y+140);
}
