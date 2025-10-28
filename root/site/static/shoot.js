import { game, GAME_OVER, COMPLETE } from "./main.js";
import { hideTarget, addTarget } from "./target.js";
import { showFeedback, updateUI } from "./ui.js";
import { player } from "./player.js";
import { block } from "./block.js"

const BG_LAYER = document.getElementById('bgLayer');
const TGT_LAYER = document.getElementById('targetsLayer');
const WRD_DISPLAY = document.getElementById('word-display');
const BULLET_HOLES = document.getElementsByClassName('bullet-circle');

const HIT = 'HIT';
const MISS = 'MISS';
const BIM = 'BIM';

export function shoot() {
    player.xAxis.classList.add('shoot');
    player.yAxis.classList.add('shoot');
    player.circle.classList.add('shoot');

    setTimeout(() => {
        player.xAxis.classList.remove('shoot');
        player.yAxis.classList.remove('shoot');
        player.circle.classList.remove('shoot');
    }, 300);  

    player.bullets--;
    if (player.bullets <= 0) game.state = GAME_OVER;

    let targetHit = false;
    let result = MISS;

    const hitBlock = (player.x >= block.x && player.x <= block.x+150) && (player.y >= block.y && player.y <= block.y+150);

    if (!hitBlock)
    game.targets.forEach((t, i) => {
        const distance = getDistance(player, t)
    
        if (distance <= t.r + 5 && t.active) {
            targetHit = true;
            hideTarget(t);
            let letter = t.letter;

            // if current word not completed
            if (result !== BIM) {
                result = checkWordCompletion(t.letter);
        
                // update stats if hit a valid target
                if (result !== MISS) {
                    player.score += Math.floor(100 * (1+(player.combo/10)));
                    player.combo++;

                    letter = game.targetList[game.currentTargetIndex];

                    game.currentTargetIndex++;
                    if (result === BIM) game.currentWordIndex++;
                    if (game.currentWordIndex >= game.wordList.length) game.state = COMPLETE;

                    showFeedback(result)
                };
            };

            setTimeout(() => {
                t = addTarget(i, letter);
            }, 1100);
        }
    });

    if (!targetHit || result === MISS){
        player.combo = 0;
        showFeedback(result);
    }
    addBulletHole(player.x, player.y, targetHit);
    updateUI();
}

function getDistance(player, t){
    const dx = player.x - t.x;
    const dy = player.y - t.y;
    return Math.sqrt(dx * dx + dy * dy);
}

// check if current word is completed and change word display accordingly
function checkWordCompletion(letter){
    const wordObj = game.wordList[game.currentWordIndex];
    let [key, value] = Object.entries(wordObj)[0];
    const word = WRD_DISPLAY.textContent;
    const parts = word.split('');
    
    let underscoreCount = 0;
    
    for (let i = 0; i < value.length; i++) {
        if (value[i] === letter){
            for (let j = 0; j < parts.length; j++) {
                if(parts[j] === ' ') underscoreCount++;

                if (underscoreCount === i+1){
                    parts[j] = letter;
                    break;
                }
            }
            WRD_DISPLAY.textContent = parts.join('');
            const index = value.indexOf(letter);
            if (index !== -1) {
                value.splice(index, 1);
            }
            game.wordList[game.currentWordIndex][key] = value;
            if (value.length === 0) return BIM;
            return HIT;
        }
    }
    return MISS
}

let bulletHoleIndex = 0;
async function addBulletHole(x, y, targetHit) {
    const layer = (targetHit) ? TGT_LAYER : BG_LAYER;
 
    bulletHoleIndex++;
    if (bulletHoleIndex >= 8) bulletHoleIndex= 0;

    const currentHole = BULLET_HOLES[bulletHoleIndex];

    currentHole.setAttribute('cx', x + (Math.random()*4)-2);
    currentHole.setAttribute('cy', y + (Math.random()*4)-2);
    currentHole.style.display = 'block';
    layer.appendChild(currentHole);
    if (targetHit) currentHole.classList.add('target-removal');

    setTimeout(() => {
        currentHole.classList.remove('target-removal');
        currentHole.style.display = 'none';
    }, 1000);

    return currentHole;
}