import { game } from "./main.js";
import { keysPressed } from "./input.js";

export const player = {
    xAxis: document.getElementById('yAxis'),
    yAxis: document.getElementById('xAxis'),
    circle: document.getElementById('playerOrigin'),
    y: 300,
    x: 400,
    r: 25,
    speed: 6,
    bullets: 15,
    score: 0,
    combo: 0
};

// Update the player's position in the SVG
export async function updatePlayerPosition() {
    if (keysPressed['ArrowUp'] || keysPressed['w']) {
        const newY = player.y - player.speed;
        player.y = (newY < player.r+50) ? player.r+50 : newY;
    };
    if (keysPressed['ArrowDown'] || keysPressed['s']) {
        const newY = player.y + player.speed;
        player.y = (newY > game.height-player.r-50) ? game.height-player.r-50 : newY;
    };
    if (keysPressed['ArrowLeft'] || keysPressed['a']) {
        const newX = player.x - player.speed;
        player.x = (newX < player.r) ? player.r : newX;
    };
    if (keysPressed['ArrowRight'] || keysPressed['d']) {
        const newX = player.x + player.speed;
        player.x = (newX > game.width-player.r) ? game.width-player.r : newX;
    };
    setPlayerPostion();
}

// Function to initialize the player
export async function initPlayer(points) {
    player.score = points;
    player.combo = 0;
    player.bullets = 15;
    player.y = 300;
    player.x = 400;
    setPlayerPostion();
}

function setPlayerPostion(){
    player.yAxis.setAttribute('y1', player.y);
    player.yAxis.setAttribute('y2', player.y);
    player.xAxis.setAttribute('x1', player.x);
    player.xAxis.setAttribute('x2', player.x);
    player.circle.setAttribute('cx', player.x);
    player.circle.setAttribute('cy', player.y);
}