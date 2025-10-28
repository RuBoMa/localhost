import { game, START, AT_START, PAUSED, AT_PAUSED, GAME_OVER, COMPLETE } from "./main.js";

const PAUSE_MENU = document.getElementById('pause-menu');
const PAUSE_BTN_TEXT = document.getElementsByClassName('pauseText');
const PAUSE_BTN_RECT = document.getElementsByClassName('pauseButton');
const START_MENU = document.getElementById('start-menu');
const GAME_OVER_DISPLAY = document.getElementById('gameover-display');
const GAME_COMPLETE_DISPLAY = document.getElementById('gamecomplete-display');

function showStartScreenMenu() {  
    START_MENU.style.display = 'block';
    START_MENU.classList.add('fade-in');
    game.state = AT_START;
}

function hideStartScreenMenu() {
    START_MENU.style.display = 'none';
    START_MENU.classList.remove('fade-in');
}

function showPauseMenu() {
    PAUSE_MENU.style.display = 'block';
    PAUSE_MENU.classList.add('fade-in');
    game.pauseSelection = 0;
    PAUSE_BTN_TEXT[0].setAttribute("fill", "green")
    PAUSE_BTN_RECT[0].setAttribute("stroke", "green")
    PAUSE_BTN_TEXT[1].setAttribute("fill", "grey")
    PAUSE_BTN_RECT[1].setAttribute("stroke", "grey")
    PAUSE_BTN_TEXT[2].setAttribute("fill", "grey")
    PAUSE_BTN_RECT[2].setAttribute("stroke", "grey")
    game.state = AT_PAUSED;
}

function hidePauseMenu(){
    PAUSE_MENU.style.display = 'none';
    PAUSE_MENU.classList.remove('fade-in');
}

function showGameOver() {                     
    GAME_OVER_DISPLAY.style.display = 'block';
    GAME_OVER_DISPLAY.classList.add('fade-in');
}

function hideGameOver() {
    GAME_OVER_DISPLAY.style.display = 'none';
    GAME_OVER_DISPLAY.classList.remove('fade-in');
}

function showGameComplete() {                     
    GAME_COMPLETE_DISPLAY.style.display = 'block';
    GAME_COMPLETE_DISPLAY.classList.add('fade-in');
}

function hideGameComplete() {
    GAME_COMPLETE_DISPLAY.style.display = 'none';
    GAME_COMPLETE_DISPLAY.classList.remove('fade-in');
}

export function hideAllMenus(){
    hideStartScreenMenu();
    hidePauseMenu();
    hideGameOver();
    hideGameComplete();
};

export function showMenu(){
    if (game.state === START) showStartScreenMenu();
    if (game.state === PAUSED) showPauseMenu();
    if (game.state === GAME_OVER) showGameOver();
    if (game.state === COMPLETE) showGameComplete();
}