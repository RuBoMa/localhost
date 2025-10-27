import { game, RUNNING, GAME_OVER } from './main.js';
import { player } from './player.js';
import { playSFX } from './audio.js';

const SCORE_DISPLAY = document.getElementById('score-display');
const COMBO_DISPLAY = document.getElementById('combo-display');
const WRD_DISPLAY = document.getElementById('word-display');
const WRD_AIM_DISPLAY = document.getElementById('wordAim-display');
const FEEDBACK_DISPLAY = document.getElementById('feedback-display');
const LOWER_DISPLAY = document.getElementById('lower-display');
const TIMER_DISPLAY = document.getElementById('timer');
const BULLET_DISPLAY = document.getElementById('bulletCount');

let timerValue;
let timerInterval;

export async function initUI(){
    initTimer();
    updateWordDisplay();
    updateScoreDisplay();
    updateBulletDisplay(player.bullets);
}

export async function updateUI(result){
    updateBulletDisplay();
    updateScoreDisplay();
}

async function initTimer(){
    TIMER_DISPLAY.setAttribute('fill', 'limegreen');
    TIMER_DISPLAY.classList.remove('flashing-red');
    clearInterval(timerInterval); 
    timerValue = game.timeDuration;
    TIMER_DISPLAY.textContent = timerValue;
    timerInterval = setInterval(updateTimer, 1000);
}

async function updateTimer() {
    if (game.state === RUNNING) {
        if (timerValue > 0) {
            if (timerValue < 10){
                TIMER_DISPLAY.classList.add('flashing-red');
            } else if (timerValue < 20){
                TIMER_DISPLAY.setAttribute('fill', 'red');
            } else if (timerValue < 40){
                TIMER_DISPLAY.setAttribute('fill', 'gold');
            }
            timerValue--;
            TIMER_DISPLAY.textContent = timerValue;
        } else {
            game.state = GAME_OVER;
            clearInterval(timerInterval);
        }
    };
}

async function updateBulletDisplay(){
    BULLET_DISPLAY.setAttribute('fill', 'limegreen');
    BULLET_DISPLAY.classList.remove('flashing-red');

    let result = '';

    for (let i = 0; i < player.bullets; i ++){
        result += '❚';
    }
    BULLET_DISPLAY.textContent = result;

    if (player.bullets === 1) {
        BULLET_DISPLAY.classList.add('flashing-red');
    } else if (player.bullets <= 3) {
        BULLET_DISPLAY.setAttribute('fill', 'red');
    } else if (player.bullets <= 7) {
        BULLET_DISPLAY.setAttribute('fill', 'gold');
    }
}

async function updateWordDisplay() {
    if (game.currentWordIndex >= game.wordList.length) return;

    // show word with missing characters
    const [word] = Object.keys(game.wordList[game.currentWordIndex]);
    WRD_DISPLAY.textContent = `[${word.replaceAll('_',' ')}]`;

    // show word with all characters
    const [key, value] = Object.entries(game.wordList[game.currentWordIndex])[0];
    let parts = key.split('');
    let valueIndex = 0;
    for (let i = 0; i < parts.length; i++){
        if (parts[i] === '_') {
            parts[i] = value[valueIndex];
            valueIndex++;
        };
    };
    WRD_AIM_DISPLAY.textContent = `[${parts.join('')}]`;
};

let timeoutID;
export async function showFeedback(result) {
    if (result === 'MISS') {
        FEEDBACK_DISPLAY.setAttribute('fill', 'red');
        LOWER_DISPLAY.classList.add('shake');
        playSFX('MISS')
        setTimeout(() => {
            LOWER_DISPLAY.classList.remove('shake');
        }, 300)
    } else {
        FEEDBACK_DISPLAY.setAttribute('fill', 'limegreen');
        LOWER_DISPLAY.classList.add('hit');
        playSFX('HIT')
        setTimeout(() => {
            LOWER_DISPLAY.classList.remove('hit');
        }, 300)
    };

    FEEDBACK_DISPLAY.textContent = result
    if (timeoutID !== undefined){
        clearTimeout(timeoutID);
        FEEDBACK_DISPLAY.style.display = 'none';
        FEEDBACK_DISPLAY.classList.remove('fade-in-out-up');
    }
    FEEDBACK_DISPLAY.setAttribute('x', player.x) 
    FEEDBACK_DISPLAY.setAttribute('y', player.y)
    setTimeout(() => {
        FEEDBACK_DISPLAY.style.display = 'block';
        FEEDBACK_DISPLAY.classList.add('fade-in-out-up');
    }, 10)
    timeoutID = setTimeout(() => {
        FEEDBACK_DISPLAY.style.display = 'none';
        FEEDBACK_DISPLAY.classList.remove('fade-in-out-up');
        timeoutID = undefined
    }, 610);
    
    if (result === 'BIM'){
        WRD_DISPLAY.setAttribute('fill', 'green');
        playSFX('BIM');
        setTimeout(() => {
            WRD_DISPLAY.setAttribute('fill', 'red');
            updateWordDisplay();
        }, 600);
    }
}

async function updateScoreDisplay(){
    if (player.combo === 0) {
        COMBO_DISPLAY.style.display = "none";
    } else {
        COMBO_DISPLAY.style.display = "block";
        COMBO_DISPLAY.textContent = `COMBO ${player.combo}`;
    }
    SCORE_DISPLAY.textContent = player.score.toString().padStart(6, '0');
}

