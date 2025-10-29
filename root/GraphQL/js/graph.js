import { formatDisplayDate, getLastPathSegment, formatMonthYear } from './utils.js';

export function createProgressLineGraph(xpTransactions, options = {}) {
    if (!xpTransactions || xpTransactions.length === 0) {
        return '<p class="no-data">No XP data available</p>';
    }

    // Sort transactions by date (oldest first)
    const sorted = [...xpTransactions].sort((a, b) => 
        new Date(a.createdAt) - new Date(b.createdAt)
    );

    // tx per day
    const grouped = groupTransactionsByDate(sorted);
    const dailyData = Object.entries(grouped).map(([date, txs]) => {
        const totalXP = txs.reduce((sum, tx) => sum + tx.amount, 0);
        const path = txs[0].path; // Use first transaction's path for marker color
        const createdAt = txs[0].createdAt; // Use first transaction's timestamp for sorting
        return { date, totalXP, path, createdAt };
    });

    // Sort daily data by date
    dailyData.sort((a, b) => new Date(a.createdAt) - new Date(b.createdAt));

    // Calculate cumulative XP
    let cumulativeXP = [];
    let runningTotal = 0;

    dailyData.forEach(day => {
        runningTotal += day.totalXP;
        cumulativeXP.push(runningTotal);
    });

    const monthLabels = [];
    let lastMonth = '';
    dailyData.forEach((day, i) => {
        const monthYear = formatMonthYear(day.createdAt);
        if (monthYear !== lastMonth) {
            monthLabels.push({ idx: i, label: monthYear });
            lastMonth = monthYear;
        }
    });


    // Config with defaults; 
    // can be overridden if "options" or a new object of config is passed
    const config = {
        width: 1000,
        height: 600,
        padding: 100,
        lineColor: '#8968CD',   
        gridColor: '#BFA980',   
        textColor: '#333',
        ...options
    };

    // Calculate Y-axis grid lines (50k increments)
    const maxXP = Math.max(...cumulativeXP);
    const yMax = Math.ceil(maxXP / 50000) * 50000;
    const gridLines = [];
    for (let y = 0; y <= yMax; y += 50000) {
        gridLines.push(y);
    }

    // --- Date-proportional X-axis calculation ---
    const firstDate = new Date(dailyData[0].createdAt);
    const lastDate = new Date(dailyData[dailyData.length - 1].createdAt);
    let totalDays = (lastDate - firstDate) / (1000 * 60 * 60 * 24); // 86,400,000 milliseconds in a day; 86,400 seconds, 1,440 minutes,
    if (totalDays === 0) totalDays = 1; // Prevent division by zero for single point

    const points = dailyData.map((day, i) => {
        const daysFromStart = (new Date(day.createdAt) - firstDate) / (1000 * 60 * 60 * 24);
        const x = config.padding + (daysFromStart / totalDays) * (config.width - 2.1 * config.padding);
        const y = config.padding + (config.height - 2 * config.padding) - 
                  (cumulativeXP[i] / yMax) * (config.height - 2 * config.padding);
        return { x, y };
    });

    const xLabelsSVG = monthLabels.map(({ idx, label }) => {
    const pt = points[idx];
    return `
        <text x="${pt.x}" y="${config.height - config.padding + 18}" 
                text-anchor="middle" font-size="12" fill="${config.textColor}">
            ${label}
        </text>
        `;
    }).join('');


    // Build colored markers for each point using getMarkerColor
    let pointsSVG = '';
    points.forEach((pt, i) => {
        const path = dailyData[i].path;
        const color = getMarkerColor(path);
        const displayDate = formatDisplayDate(dailyData[i].date);
        const displayPath = path.includes('checkpoint') 
            ? 'checkpoint'
            : getLastPathSegment(path);
        
        const tooltipWidth = 125;
        const tooltipHeight = 55;
        const tooltipOffset = 8; // space between marker and tooltip
        const markerX = pt.x - tooltipWidth/2 - tooltipOffset - 38; // small left padding in tooltip
        const textX = markerX + 12; // 14px to the right of the circle

        pointsSVG += `
        <g class="marker-group" tabindex="0">
            <circle cx="${pt.x}" cy="${pt.y}" r="12" fill="transparent" pointer-events="all"/>
            <circle cx="${pt.x}" cy="${pt.y}" r="5" fill="${color}" stroke="#fff" stroke-width="0.5"/>
            <g class="svg-tooltip" style="pointer-events:none;">
                <rect x="${pt.x - tooltipWidth - tooltipOffset}" y="${pt.y - tooltipHeight/2}" width="${tooltipWidth}" height="${tooltipHeight}" rx="8"/>
                <text x="${pt.x - tooltipWidth/2 - tooltipOffset}" y="${pt.y - 10}" text-anchor="middle" font-size="10" fill="#333">Date: ${displayDate}</text>
                <g>
                    <circle cx="${markerX}" cy="${pt.y + 3}" r="6" fill="${color}" stroke="#fff" stroke-width="0.5"/>
                    <text x="${textX}" y="${pt.y + 5}" font-size="10" fill="#333">
                    ${displayPath}
                    </text>
                </g>           
                <text x="${pt.x - tooltipWidth/2 - tooltipOffset -15}" y="${pt.y + 22}" text-anchor="middle" font-size="10" fill="#333">XP: ${dailyData[i].totalXP.toLocaleString()}</text>
            </g>
        </g>
        `;
    });

    const legendWidth = 325;
    const legendX = (config.width - legendWidth) / 2;
    const legendY = config.height - 23; // move further down if needed

    const legend = `
    <g transform="translate(${legendX},${legendY})">
        <rect x="0" y="0" width="${legendWidth}" height="30" rx="8" fill="none" stroke="none"/>
        <circle cx="29" cy="15" r="7" fill="#FFB7B2" stroke="#fff" stroke-width="0.5"/> 
        <text x="40" y="20" font-size="14" fill="${config.textColor}">Checkpoint</text>
        <circle cx="139" cy="15" r="7" fill="#BFA2DB" stroke="#fff" stroke-width="0.5"/>
        <text x="150" y="20" font-size="14" fill="${config.textColor}">Piscine</text>
        <circle cx="219" cy="15" r="7" fill="#AEEBFF" stroke="#fff" stroke-width="0.5"/>
        <text x="230" y="20" font-size="14" fill="${config.textColor}">Project</text>
    </g>
    `;

    let svg = `
        <svg width="${config.width}" height="${config.height}" viewBox="0 0 ${config.width} ${config.height}">
            <!-- Title -->
            <text x="${config.width / 2}" y="20" text-anchor="middle" font-size="28" font-weight="bold" fill="${config.textColor}">
            XP Progress Over Time
            </text>
            <!-- Y-axis grid lines -->
            ${gridLines.map(yVal => {
                const y = config.padding + (config.height - 2 * config.padding) - (yVal / yMax) * (config.height - 2 * config.padding);
                return `
                    <line x1="${config.padding}" y1="${y}" x2="${config.width - config.padding}" y2="${y}" stroke="${config.gridColor}"/>
                    <text x="${config.padding - 10}" y="${y + 5}" text-anchor="end" font-size="12" fill="${config.textColor}">
                        ${yVal.toLocaleString()}
                    </text>
                `;
            }).join('')}

            <!-- X-axis line -->
            <line x1="${config.padding}" y1="${config.height - config.padding}" x2="${config.width - config.padding}" y2="${config.height - config.padding}" stroke="${config.textColor}"/>
            <!-- X-axis labels -->
            ${xLabelsSVG}

            <!-- Line path -->
            <path d="${points.map((pt, i) => (i === 0 ? `M${pt.x},${pt.y}` : `L${pt.x},${pt.y}`)).join(' ')}" 
                    fill="none" 
                    stroke="${config.lineColor}" 
                    stroke-width="2"/>

            <!-- Colored Data points (one per day) -->
            ${pointsSVG}
            <!-- Legend -->
            ${legend}
        </svg>
    `;

    return svg;
}

// helper func for LineGraph
function groupTransactionsByDate(transactions) {
    const grouped = {};
    transactions.forEach(tx => {
        const date = tx.createdAt.split('T')[0]; // "YYYY-MM-DD"
        if (!grouped[date]) {
            //creating new arr for date
            grouped[date] = [];
        }
        grouped[date].push(tx);
    });
    // returning the object of arrays of the "date"
    return grouped;
}

function getMarkerColor(path) {
    if (path.includes('checkpoint')) {
    return '#FFB7B2'; 

    }
    if (path.includes('piscine-')) {
        return '#BFA2DB'; 
    }
    return '#AEEBFF'; 

}

// --- First Bar Graph --- //

export function createAuditBarGraph(auditData, options = {}) {
    // auditData: { totalUp, totalDown }
    // options: { width, height, barColors, ... }

    const config = {
        width: 300,
        height: 180,
        barColors: ['#D69A6D', '#A3A8A3'],
        ...options
    };

    const labels = ['Done XP', 'Received XP'];
    const values = [
        auditData.totalUp,
        auditData.totalDown
    ];

    const maxValue = Math.max(...values) * 1.1; // Add 11% headroom

    const barWidth = config.width / (values.length * 2);
    const gap = barWidth / 2;

    let bars = '';
    values.forEach((val, i) => {
        const barHeight = (val / maxValue) * (config.height - 40); // leave space for labels
        const x = gap + i * (barWidth + gap);
        const y = config.height - barHeight - 20; // leave space for x labels
        bars += `
            <rect x="${x}" y="${y}" width="${barWidth}" height="${barHeight}" fill="${config.barColors[i % config.barColors.length]}" rx="6"/>
            <text x="${x + barWidth / 2}" y="${config.height - 5}" text-anchor="middle" font-size="12" fill="#4B3B53">${labels[i]}</text>
            <text x="${x + barWidth / 2}" y="${y - 5}" text-anchor="middle" font-size="12" fill="#4B3B53">${val}</text>
        `;
    });

    return `
        <svg width="${config.width}" height="${config.height}" viewBox="0 0 ${config.width} ${config.height}" aria-label="Audit Bar Graph">
            ${bars}
        </svg>
    `;
}

// --- Second Bar Graph --- //

export function createProjectBarGraph(xpTransaction, options = {}) {
// Aggregate XP by project (excluding piscine/checkpoint)
    const projectXP = {};
    xpTransaction.forEach(tx => {
        const pathLower = tx.path.toLowerCase();
        if (!pathLower.includes('piscine') && !pathLower.includes('checkpoint')) {
        const project = getLastPathSegment(tx.path);
        if (!projectXP[project]) projectXP[project] = 0;
        projectXP[project] += tx.amount;
        }
    });

    const labels = Object.keys(projectXP);
    const values = Object.values(projectXP);

    labels.reverse();
    values.reverse();

    if (!labels.length) return '<p>No project XP data available.</p>';

    const config = {
        width: 850,
        height: 400,
        barColor: '#B76E79',
        ...options
    };
    const topMargin = 120;    // space for rotated labels above bars
    const bottomMargin = 30; // space at bottom for XP values
    const barAreaHeight = config.height - topMargin - bottomMargin;

    const barWidth = (config.width - 60) / labels.length;
    const maxValue = Math.max(...values) * 1.1;

    let bars = '';

    labels.forEach((label, i) => {
        const barHeight = (values[i] / maxValue) * barAreaHeight;
        const x = 40 + i * barWidth;
        const y = config.height - barHeight - bottomMargin; // position from bottom
        bars += `
        <rect x="${x}" y="${y}" width="${barWidth - 10}" height="${barHeight}" fill="${config.barColor}" rx="5"/>
        <text x="${x + (barWidth - 10) / 2}" y="${y -25}" text-anchor="end" font-size="12" fill="#333" 
        transform="rotate(-270 ${x + (barWidth - 10) / 2} ${y - 25})">${label}</text>
        <text x="${x + (barWidth - 10) / 2}" y="${y - 8}" text-anchor="middle" font-size="12" fill="#333">${values[i]}</text>
        `;
    });

    return `
        <svg width="100%" height="${config.height}" viewBox="0 0 ${config.width} ${config.height}" aria-label="XP by Project">
        ${bars}
        </svg>
    `;
}
