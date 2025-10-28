const rootStyles = getComputedStyle(document.documentElement)
const primaryColor = `hsl(${rootStyles.getPropertyValue('--primary-color').trim()})`
const secondaryColor = `hsl(${rootStyles.getPropertyValue('--secondary-color').trim()})`
const tertiaryColor = `hsl(${rootStyles.getPropertyValue('--tertiary-color').trim()})`

const column = 10
const row = 20
const squarePiece = [[0, 4], [0, 5], [1, 4], [1, 5]];
const tallPiece = [[0, 3], [0, 4], [0, 5], [0, 6]];
const lightningPiece = [[0, 4], [0, 5], [1, 5], [1, 6]];
const reverseLightningPiece = [[1, 4], [1, 5], [0, 5], [0, 6]];
const lPiece = [[1, 4], [1, 5], [1, 6], [0, 6]];
const reverseLPiece = [[1, 4], [1, 5], [1, 6], [0, 4]];
const tPiece = [[1, 4], [1, 5], [1, 6], [0, 5]];
const allPieces = [squarePiece, tallPiece, lightningPiece, reverseLightningPiece, lPiece, reverseLPiece, tPiece]

let baseHue = 175
const baseSaturation = 75
const baseLightness = 50
const originalColor = [baseHue, baseSaturation, baseLightness]
let baseColor = [baseHue, baseSaturation, baseLightness]
let allColors = assignBaseColors()

const timeContainer = document.getElementById("timeContainer")
const scoreContainer = document.getElementById("scoreContainer")
const levelContainer = document.getElementById("levelContainer")
const timerDiv = document.createElement("div")
const menuDiv = document.getElementById("menu")
const scoreDiv = document.createElement("div")
const previewContainer = document.getElementById("previewContainer")
const gameContainer = document.getElementById("gameContainer")

let animating = false
let alive = true
let level = 1;
let score = 0;
let [piece, color, pieceType] = generatePiece()
let nextPiece = generatePiece()
let gameBoard = createGameBoard()
let previewBoard = createPreviewBoard()
let isFirstShow = true
const defaultDelay = 30
let delay = defaultDelay
let startTime = Date.now() / 1000
let lastTime = 0
let currentTime = 0
let nextDirection = "down"
let timer = 0
let showMenu = false
let oldBoard

function createGameBoard() {
  let gameBoard = new Array(row)
  let cellIndex = 0
  for (let i = 0; i < row; i++) {
    gameBoard[i] = new Array(column)
    for (let j = 0; j < column; j++) {
      gameBoard[i][j] = { full: false, color: secondaryColor, cellIndex: cellIndex }
      cellIndex++
    }
  }
  return gameBoard
}
function createPreviewBoard() {
  let previewBoard = new Array(4)
  let cellIndex = 0
  for (let i = 0; i < 5; i++) {
    previewBoard[i] = new Array(4)
    for (let j = 0; j < 5; j++) {
      previewBoard[i][j] = { full: false, color: secondaryColor, cellIndex: 20 * 10 + cellIndex }
      cellIndex++
    }
  }
  return previewBoard
}

function showBoard() {
  let screen = document.createElement("div")
  let index = 0
  screen.style.width = `${column * 2}vw`
  screen.style.height = `${row * 2}vw`
  screen.style.alignContent = "center"
  for (let i = 0; i < gameBoard.length; i++) {
    let currentRow = document.createElement("span")
    for (let j = 0; j < gameBoard[i].length; j++) {
      let cell = document.createElement("div")
      if (gameBoard[i][j].full) {
        cell.style.backgroundColor = gameBoard[i][j].color
        cell.setAttribute("cell-color", cell.color)
      } else {
        cell.style.backgroundColor = secondaryColor
        cell.setAttribute("cell-color", cell.color)
      }
      cell.style.width = "2vw"
      cell.style.minWidth = "20px"
      cell.style.maxWidth = "40px"
      cell.style.height = "2vw"
      cell.style.minHeight = "20px"
      cell.style.maxHeight = "40px"
      cell.style.outlineStyle = "solid"
      cell.style.outlineWidth = "2px"
      cell.style.outlineColor = secondaryColor
      cell.id = `cell-${index}`
      currentRow.style.display = "flex"
      currentRow.appendChild(cell)
      index++
    }
    screen.appendChild(currentRow)
  }

  let gameContainer = document.getElementById("gameContainer")
  gameContainer.appendChild(screen)
}

function showNext() {
  let previewNext = document.createElement("div")
  let index = 0
  for (let i = 0; i < 5; i++) {
    let currentRow = document.createElement("span")
    for (let j = 0; j < 5; j++) {
      let cell = document.createElement("div")
      cell.style.backgroundColor = secondaryColor
      cell.setAttribute("cell-color", cell.color)
      cell.style.width = "2vw"
      cell.style.minWidth = "20px"
      cell.style.maxWidth = "40px"
      cell.style.height = "2vw"
      cell.style.minHeight = "20px"
      cell.style.maxHeight = "40px"
      cell.style.outlineStyle = "solid"
      cell.style.outlineWidth = "2px"
      cell.style.outlineColor = secondaryColor
      cell.id = `cell-${row * column + index}`
      currentRow.style.display = "flex"
      currentRow.appendChild(cell)
      index++
    }
    previewNext.appendChild(currentRow)
  }
  previewContainer.appendChild(previewNext)
}

function updateVisuals(newPiece) {
  if (newPiece === undefined) {
    for (let i = linesToRemove[linesToRemove.length - 1]; i >= 0; i--) {
      gameBoard[i].forEach(currCell => {
        let cell = document.getElementById(`cell-${currCell.cellIndex}`)
        if (!currCell.full) {
          cell.style.backgroundColor = secondaryColor
          cell.setAttribute("cell-color", secondaryColor)
        } else {
          cell.style.backgroundColor = currCell.color
          cell.setAttribute("cell-color", currCell.color)
        }
      })
    }
  } else {
    for (let i = 0; i < piece.length; i++) {
      let row = piece[i][0];
      let col = piece[i][1];
      let cell = document.getElementById(`cell-${gameBoard[row][col].cellIndex}`);
      cell.style.backgroundColor = secondaryColor
      cell.setAttribute("cell-color", secondaryColor)
    }
    for (let i = 0; i < newPiece.length; i++) {
      let row = newPiece[i][0];
      let col = newPiece[i][1];
      let currCell = gameBoard[row][col];
      let cell = document.getElementById(`cell-${currCell.cellIndex}`);
      cell.style.backgroundColor = currCell.color;
      cell.setAttribute("cell-color", currCell.color);
    }
  }
}

function convertTimes(currTime) {
  let hours = Math.floor(currTime / (60 * 60))
  let minutes = Math.floor(currTime / (60)) - (hours * 60)
  let seconds = currTime - (minutes * 60) - (hours * 60 * 60)
  let output = ""

  if (hours.toString().length < 2) {
    output += `0${hours.toString()}:`
  } else {
    output += `${hours.toString()}:`
  }

  if (minutes.toString().length < 2) {
    output += `0${minutes.toString()}:`
  } else {
    output += `${minutes.toString()}:`
  }

  if (seconds.toString().length < 2) {
    output += `0${seconds.toString()}`
  } else {
    output += seconds.toString()
  }
  return output
}

function updateVisualPreview() {
  previewBoard.forEach(currRow => {
    currRow.forEach(currCell => {
      let cell = document.getElementById(`cell-${currCell.cellIndex}`)
      cell.style.backgroundColor = secondaryColor
      cell.setAttribute("cell-color", secondaryColor)
    })
  })

  let adjustedNextPiece = offsetPiece(nextPiece[0], 1, -3)
  for (let i = 0; i < previewBoard.length; i++) {
    for (let j = 0; j < previewBoard[0].length; j++) {
      for (let [x, y] of adjustedNextPiece) {
        if (i === x && j === y) {
          let cell = document.getElementById(`cell-${previewBoard[i][j].cellIndex}`)
          cell.style.backgroundColor = nextPiece[1]
          cell.setAttribute("cell-color", nextPiece[1])
        }
      }
    }
  }
}

function showTimer() {
  timerDiv.id = "timer"
  timerDiv.textContent = `Playtime: 00:00:00`
  timerDiv.style.color = primaryColor
  timeContainer.appendChild(timerDiv)
}

function updateTimer() {
  let currTime = Math.floor(Date.now() / 1000 - startTime)
  if (lastTime !== currTime) {
    lastTime = currTime
    let fullTime = convertTimes(currTime)
    timerDiv.textContent = `Playtime: ${fullTime}`
  }
}

function offsetPiece(fakePiece, dx, dy) {
  return fakePiece.map(([x, y]) => [x + dx, y + dy]);
}

function updatePiece(piece, dx, dy) {
  let newPiece = offsetPiece(piece, dx, dy);
  // Check if move is valid
  for (let [x, y] of newPiece) {
    if (x < 0 || x >= row || y < 0 || y >= column || gameBoard[x][y].full) {
      return [piece, dx === 1];
    }
  }
  return [newPiece, false];
}

function movePiece(direction) {
  switch (direction) {
    case "down":
      return updatePiece(piece, 1, 0);
    case "right":
      return updatePiece(piece, 0, 1);
    case "left":
      return updatePiece(piece, 0, -1);
    case "clockwise":
      return rotate("clockwise");
    case "counter clockwise":
      return rotate("counter clockwise");
  }
}


function calculatePositionRelativeToPivot(piece, pivotX, pivotY) {
  return piece.map(([x, y]) => {
    let relX = x - pivotX;
    let relY = y - pivotY;
    return [relX, relY];
  });
}

function isSquare() {
  let pieceX = piece[0][0];
  let pieceY = piece[0][1];
  if ((pieceY + 1 === piece[1][1] && pieceX === piece[1][0]) && (pieceX + 1 === piece[2][0] && pieceY === piece[2][1]) && (pieceY + 1 === piece[3][1] && pieceX + 1 === piece[3][0])) {
    return true;
  }
  return false;
}

function rotate(direction) {
  const pivot = piece[1];
  const [px, py] = pivot;
  if (isSquare()) {
    return [piece, false];
  }
  const rotateFunc = direction === "clockwise"
    ? ([x, y]) => [y, -x]
    : ([x, y]) => [-y, x];

  let rotatedPiece = piece.map(([x, y]) => {
    let [relX, relY] = [x - px, y - py];
    let [newX, newY] = rotateFunc([relX, relY]);
    return [newX + px, newY + py];
  });

  if (!isCollision(rotatedPiece)) return [rotatedPiece, false];

  const wallKicks = [[0, 0], [1, 0], [-1, 0], [0, 1], [0, -1]];
  for (let [dx, dy] of wallKicks) {
    let kickedPiece = rotatedPiece.map(([x, y]) => [x + dx, y + dy]);
    if (!isCollision(kickedPiece)) return [kickedPiece, false];
  }

  return [piece, false];
}

function isCollision(testPiece) {
  return testPiece.some(([x, y]) => x < 0 || x >= row || y < 0 || y >= column || gameBoard[x][y].full);
}

function updateBoard(direction) {
  let output = []
  for (let i = 0; i < piece.length; i++) {
    gameBoard[piece[i][0]][piece[i][1]].full = false
    gameBoard[piece[i][0]][piece[i][1]].color = secondaryColor
    gameBoard[piece[i][0]][piece[i][1]].pieceType = pieceType;
  }
  output = movePiece(direction)
  let newPiece = output[0]
  for (let i = 0; i < newPiece.length; i++) {
    gameBoard[newPiece[i][0]][newPiece[i][1]].full = true
    gameBoard[newPiece[i][0]][newPiece[i][1]].color = color
    gameBoard[newPiece[i][0]][newPiece[i][1]].pieceType = pieceType;
  }
  if (!isFirstShow) {
    updateVisuals(newPiece)
    updateTimer()
  }
  return output
}

function generatePiece() {
  let pieceInd = Math.floor(Math.random() * (allPieces.length)) // rand is between 0 (inclusive), and 1 (exclusive)
  return ([allPieces[pieceInd], allColors[pieceInd], pieceInd])
}

function removeLine(lines) {
  if (!Array.isArray(lines)) lines = [lines]; // Ensure it's an array
  if (lines.length === 0) return;
  animating = true
  currentTime = 0
  for (let line of lines) {
    gameBoard[line].forEach(elem => {
      let cell = document.getElementById(`cell-${elem.cellIndex}`)
      cell.classList.add("clearedCell")
      addEventListener("animationend", () => {
        cell.classList.remove("clearedCell")
      }, { once: true })
    });
    for (let i = line; i > 0; i--) {
      for (let j = 0; j < column; j++) {
        gameBoard[i][j].full = gameBoard[i - 1][j].full;
        gameBoard[i][j].color = gameBoard[i - 1][j].color;
      }
    }
  }
  setTimeout(() => {
    animating = false
    // Make sure the top row is cleared after shifting
    for (let j = 0; j < column; j++) {
      gameBoard[0][j].full = false;
      gameBoard[0][j].color = secondaryColor;
    }
    requestAnimationFrame(() => updateVisuals())
  }, 500)
}

function findContinuousLines(linesToRemove) {
  let result = [];
  let startIndex = 0;
  let continuousLinesCounter = 1;

  if (linesToRemove.length === 1) {
    return [linesToRemove];
  }

  for (let i = 1; i < linesToRemove.length; i++) {
    if (linesToRemove[i] - linesToRemove[i - 1] === 1) {
      continuousLinesCounter++;
    } else {
      result.push(linesToRemove.slice(startIndex, startIndex + continuousLinesCounter));
      startIndex += continuousLinesCounter;
      continuousLinesCounter = 1;
    }

    if (i === linesToRemove.length - 1) {
      result.push(linesToRemove.slice(startIndex, startIndex + continuousLinesCounter));
    }
  }

  return result;
}

let linesToRemove

function checkForlines() {
  linesToRemove = findLines();
  if (!linesToRemove.length) return;
  linesToRemove.sort((a, b) => a - b);
  requestAnimationFrame(() => removeLine(linesToRemove))
  let categorizedLines = findContinuousLines(linesToRemove);

  for (let i = 0; i < categorizedLines.length; i++) {
    scoreCalculator(categorizedLines[i].length);
  }
}

function findLines() {
  let wholeLine = false;
  let linesToRemove = [];
  let rowsArray = piece.map(cell => cell[0]);
  let rows = new Set();
  rowsArray.forEach(row => {
    rows.add(row);
  });
  rows.forEach(row => {
    wholeLine = true;
    for (let j = 0; j < column; j++) {
      if (!gameBoard[row][j].full) {
        wholeLine = false;
        break;
      }
    }
    if (wholeLine) {
      linesToRemove.push(row)
    }
  });
  return linesToRemove;
}

function openCloseMenu() {
  if (showMenu) {
    menuDiv.style.display = "none"
  } else {
    menuDiv.style.display = "flex"
  }
  showMenu = !showMenu
}

function showScore() {
  scoreDiv.id = "score";
  scoreDiv.textContent = `Score: ${score}`;
  scoreDiv.style.color = primaryColor;
  scoreContainer.appendChild(scoreDiv);
}

function updateScore() {
  scoreDiv.textContent = `Score: ${score}`;
}

function scoreCalculator(numberOfLines) {
  switch (numberOfLines) {
    case 1:
      score += level * 100;
      break;
    case 2:
      score += level * 300;
      break;
    case 3:
      score += level * 500;
      break;
    case 4:
      score += level * 800;
      break;
  }
}

function reset() {
  gameContainer.innerHTML = "";
  previewContainer.innerHTML = "";
  levelContainer.textContent = "Level: 1";
  gameBoard = createGameBoard();
  previewBoard = createPreviewBoard();
  [piece, color, pieceType] = generatePiece();
  nextPiece = generatePiece();
  alive = true;
  isFirstShow = true;
  currentTime = 0;
  nextDirection = "down";
  startTime = Date.now() / 1000;
  lastTime = 0;
  score = 0;
  level = 1;
  delay = defaultDelay;
  baseColor = [...originalColor];
  allColors = assignBaseColors()
  openCloseMenu();
}

function assignBaseColors() {
  return [`hsl(${baseColor[0] + 99}, ${baseColor[1] + 25}%, ${baseColor[2] + 18}%)`, // squarePiece color
  `hsl(${baseColor[0] + 81}, ${baseColor[1] + 25}%, ${baseColor[2] + 15}%)`, // tallPiece color
  `hsl(${baseColor[0] + 63}, ${baseColor[1] + 25}%, ${baseColor[2] + 12}%)`, // lightningPiece color
  `hsl(${baseColor[0] + 45}, ${baseColor[1] + 25}%, ${baseColor[2] + 9}%)`, // reverseLightningPiece color
  `hsl(${baseColor[0] + 27}, ${baseColor[1] + 25}%, ${baseColor[2] + 6}%)`, // lPiece color
  `hsl(${baseColor[0] + 9}, ${baseColor[1] + 25}%, ${baseColor[2] + 3}%)`, // reverseLPiece color
  `hsl(${baseColor[0] - 9}, ${baseColor[1] + 25}%, ${baseColor[2]}%)`] // tPiece color
}

function updateColors() {
  baseHue += 50;
  if (baseHue >= 360) {
    baseHue -= 360;
  }
  baseColor[0] = baseHue;
  allColors = assignBaseColors()
  color = allColors[pieceType];
  nextPiece[1] = allColors[nextPiece[2]];
  for (let i = 0; i < row; i++) {
    for (let j = 0; j < column; j++) {
      let cell = gameBoard[i][j];
      if (cell.full && typeof cell.pieceType === "number") {
        cell.color = allColors[cell.pieceType];
        let cellElem = document.getElementById(`cell-${cell.cellIndex}`);
        if (cellElem) {
          cellElem.style.backgroundColor = cell.color;
        }
      }
    }
  }
}

function updateLevel() {
  let newLevel = Math.floor(score / 1000) + 1;
  if (newLevel > level) {
    level = newLevel;
    updateColors()
    levelContainer.textContent = `Level: ${level}`
    if (delay > 0) {
      delay -= Math.ceil(15 / level);
    } else {
      delay = 1
    }
  }
}

function gameLoop() {
  if (!showMenu && alive && !animating) {
    let direction = nextDirection
    if (direction !== "down") {
      oldBoard = gameBoard.map(row => row.map(cell => ({ ...cell })))
      if (direction === "user down") {
        score++
        direction = "down"
      }
      let pieceAndBool = updateBoard(direction, oldBoard)
      if (!pieceAndBool[1]) {
        piece = pieceAndBool[0]
      } else {
        checkForlines(oldBoard);
        [piece, color, pieceType] = nextPiece
        nextPiece = generatePiece()
        if (isCollision(piece)) {
          showMenu = false
          alive = false
          openCloseMenu()
        }
      }
      nextDirection = "down"
    } else if (currentTime > delay) {
      oldBoard = gameBoard.map(row => row.map(cell => cell))
      let pieceAndBool = updateBoard("down", oldBoard)
      if (!pieceAndBool[1]) {
        piece = pieceAndBool[0]
      } else {
        checkForlines(oldBoard);
        [piece, color, pieceType] = nextPiece
        nextPiece = generatePiece()
        if (isCollision(piece)) {
          showMenu = false
          alive = false
          openCloseMenu()
        }
        nextDirection = "down"
      }
      currentTime -= delay
    }
    if (isFirstShow) {
      showBoard()
      showNext()
      showTimer()
      showScore()
      isFirstShow = false
    } else {
      updateScore()
      updateVisualPreview()
    }
    currentTime++
  }
  updateLevel()
  requestAnimationFrame(gameLoop)
}

addEventListener("keydown", (event) => {
  if (!animating) {
    switch (event.key) {
      case "ArrowRight":
        nextDirection = "right"
        break;
      case "ArrowLeft":
        nextDirection = "left"
        break;
      case "ArrowDown":
        nextDirection = "user down";
        break;
      case "x":
        nextDirection = "clockwise";
        break;
      case "z":
        nextDirection = "counter clockwise";
        break;
      case "Escape":
        openCloseMenu();
    }
  }
})

addEventListener("click", (event) => {
  if (event.target.id === "restart") {
    reset();
  } else if (event.target.id === "continue") {
    openCloseMenu();
  }
})

requestAnimationFrame(gameLoop)
