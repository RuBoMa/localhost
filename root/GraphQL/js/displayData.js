import { fetchAllData } from './fetchData.js';
import { createProgressLineGraph, createAuditBarGraph, createProjectBarGraph} from './graph.js';
import { formatDisplayDate, getLastPathSegment } from './utils.js';

export async function renderData() {
    try {
        const { user, xpSum, xpTransaction, typeTransaction, lastAudit } = await fetchAllData();

        // Populate user info
        // Populate tables
        // Use createProgressLineGraph(data.xpTransaction) to create SVG and insert into DOM
        // ...etc...

        displayUserData(user);
        displayAuditInfo(user, typeTransaction, lastAudit);
        displayTotalXP(xpSum);

        displayProgressLineGraph(xpTransaction);
        displayAuditBarGraph(user);
        displayAuditRatioNum(user);
        displayProjectBarGraph(xpTransaction);

    } catch (err) {
        console.error('Error fetching data:', err)
        throw err;
    }
}

// Display data on dashboard cards
function displayUserData(userData) {
    if (!userData) return;

    const userDataDiv = document.getElementById('userData');
    userDataDiv.innerHTML = `
        <p>Name: ${userData.firstName} ${userData.lastName}</p>
        <p>LoginID: ${userData.login}</p>
        <p>ID: ${userData.id}</p>
        <p>Nationality: ${userData.attrs.nationality}</p>
        <p>Campus: ${userData.campus}</p>
    `;

    const userAvatarDiv = document.getElementById('userAvatar');
    // Set your static avatar image here
    userAvatarDiv.innerHTML = `
        <img src="assets/userAvatar.jpeg" alt="My Avatar" class="avatar-img">
    `;
}

function displayAuditInfo(user, typeTransaction, lastAudit) {
    const auditType = document.getElementById('auditType')

    const totalUp = typeTransaction.filter(t => t.type === 'up').length;
    const totalDown = typeTransaction.filter(t => t.type === 'down').length;
    const total = totalUp + totalDown;

    auditType.innerHTML = `
        <p>Total Audits: ${total}</p>
        <p>Audits Given: ${totalUp}</p>
        <p>Audits Received: ${totalDown}</p>
        <h2>Latest Audit</h2>
        <p>Date: ${lastAudit?.updatedAt ? formatDisplayDate(lastAudit.updatedAt) : '-'}</p>
        <p>Auditor: ${lastAudit?.auditorLogin || '-'}</p>
        <p>Captain: ${lastAudit?.group.captainLogin || '-'}</p>
        <p>Project: ${lastAudit?.group?.path? getLastPathSegment(lastAudit.group.path): '-'}</p>
    `;
}

function displayTotalXP(xpSum) {
    const xpDiv = document.getElementById('dataInfo');
    xpDiv.innerHTML = `
    <h2>Experience Total: ${xpSum}</h2>
    `;
}
// Line graph for xp prog //

function displayProgressLineGraph(xpTransaction) {
    const graphExpDiv = document.getElementById('graphExp');

    if (!graphExpDiv) return;
    graphExpDiv.innerHTML = createProgressLineGraph(xpTransaction, {
        // Optional: Override defaults here
        width: 1000,
        height: 600,
        padding: 100,
    });
}

// Bar Graph //
function displayAuditRatioNum(user) {
    const auditRatioBigDiv = document.getElementById('auditRatioBig');
    if (!auditRatioBigDiv) return;

    let auditRatio = user.auditRatio;
    let displayHTML;

    try {
        // Handle null, empty, or zero values, or if conversion to number fails
        if (
            auditRatio === null ||
            auditRatio === '' ||
            isNaN(Number(auditRatio)) ||
            Number(auditRatio) === 0
        ) {
            displayHTML = `
                <span style="font-size:4rem;color:#F7F7F7;">N/A</span>
                <span style="font-size:1.2rem;color:#4B3B53;">Audit Ratio</span>
            `;
        } else {
            const roundedAuditRatio = Number(auditRatio).toFixed(1);
            displayHTML = `
                <span style="font-size:4rem;font-weight:bold;color:#F7F7F7;">
                    ${roundedAuditRatio}
                </span>
                <span style="font-size:1.2rem;color:#4B3B53;margin-left:12px;">
                    Audits Ratio
                </span>
            `;
        }
    } catch (e) {
        // Fallback for any unexpected errors
        displayHTML = `
            <span style="font-size:4rem;color:#F7F7F7;">N/A</span>
            <span style="font-size:1.2rem;color:#4B3B53;">Audit Ratio</span>
        `;
    }

    auditRatioBigDiv.innerHTML = displayHTML;
}

function displayAuditBarGraph(user) {
    const auditData = {
        auditRatio: parseFloat(user.auditRatio).toFixed(1),
        totalUp: user.totalUp,
        totalDown: user.totalDown
    };
    const barGraphSVG = createAuditBarGraph(auditData);
    document.getElementById('graphBar').innerHTML = barGraphSVG;
}

export function displayProjectBarGraph(xpTransaction) {
    const rowD = document.getElementById('rowD');
    if (!rowD) return;
    rowD.innerHTML = `
        <h2 style="margin-bottom: 16px;">XP by Project</h2>
        ${createProjectBarGraph(xpTransaction, { barColor: '#B76E79' })}
    `;
}
