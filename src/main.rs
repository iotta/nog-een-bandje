use axum::{
    extract::{Query, State},
    http::{header, StatusCode},
    response::{Html, IntoResponse, Json},
    routing::get,
    Router,
};
use once_cell::sync::Lazy;
use rand::seq::IndexedRandom;
use serde::{Deserialize, Serialize};
use std::fs;
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};

// --- Data Structures ---

// Structs to parse the initial JSON data from the file.
#[derive(Debug, Deserialize)]
struct BandData {
    festivals: Vec<Festival>,
}

#[derive(Debug, Deserialize)]
struct Festival {
    name: String,
    years: Vec<FestivalYear>,
}

#[derive(Debug, Deserialize)]
struct FestivalYear {
    year: u16,
    artists: Vec<String>,
}

// A new, flattened struct to hold performance details.
// This is easier to work with and will be used for all API responses.
#[derive(Debug, Clone, Serialize)]
struct ArtistPerformance {
    name: String,
    festival: String,
    year: u16,
}

// --- Application State ---
// This struct holds the flattened list of all performances, ready for any operation.
#[derive(Debug)]
struct AppState {
    all_performances: Vec<ArtistPerformance>,
}

// Use Lazy to read and process the file only once at application startup.
static APP_STATE: Lazy<Arc<AppState>> = Lazy::new(|| {
    println!("Loading bands.json into memory...");

    // Read the JSON file from the project root.
    let file_content = fs::read_to_string("bands.json")
        .expect("Failed to read bands.json. Make sure the file is in the project root.");

    // Parse the JSON into our Rust structs.
    let band_data: BandData =
        serde_json::from_str(&file_content).expect("Failed to parse bands.json.");

    // Flatten the nested structure into a single list of `ArtistPerformance` objects.
    let mut all_performances = Vec::new();
    for festival in band_data.festivals {
        for year in festival.years {
            for artist in year.artists {
                all_performances.push(ArtistPerformance {
                    name: artist,
                    festival: festival.name.clone(),
                    year: year.year,
                });
            }
        }
    }

    let performance_count = all_performances.len();
    println!(
        "Successfully loaded {} total artist performances.",
        performance_count
    );

    // Store the final list in our shared state, wrapped in an Arc for thread-safety.
    Arc::new(AppState { all_performances })
});

// --- Query Parameters for the API Request ---

#[derive(Debug, Deserialize)]
struct RandomBandParams {
    count: Option<usize>,
}

// --- Main Application Entry Point ---

#[tokio::main]
async fn main() {
    // Set up a permissive CORS layer, allowing requests from any origin.
    let cors = CorsLayer::new().allow_origin(Any).allow_methods(Any);

    // Build our application router.
    let app = Router::new()
        // Serves the main HTML interface.
        .route("/", get(root_handler))
        // API endpoint for getting random bands.
        .route("/api/random-bands", get(random_bands_api_handler))
        // New API endpoint for downloading all band data.
        .route("/api/all-bands", get(all_bands_handler))
        .layer(cors)
        .with_state(Arc::clone(&APP_STATE));

    // Define the address and port to run the server on.
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    println!("->> LISTENING on http://{}\n", addr);
    println!("->> UI available at:           http://{}", addr);
    println!(
        "->> Download API available at: http://{}/api/all-bands",
        addr
    );
    println!(
        "->> Randomizer API available at: http://{}/api/random-bands?count=3\n",
        addr
    );

    // Run the server.
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

// --- HTML Page Handler ---

async fn root_handler() -> Html<&'static str> {
    // Serve the static HTML content.
    Html(HTML_PAGE)
}

// --- API Handlers ---

/// API handler for providing a random selection of artist performances.
async fn random_bands_api_handler(
    State(state): State<Arc<AppState>>,
    Query(params): Query<RandomBandParams>,
) -> impl IntoResponse {
    // Clamp the requested count between 1 and 5. Default to 1 if not provided.
    let count = params.count.unwrap_or(1).clamp(1, 5);
    let mut rng = rand::rng();

    // Choose multiple random performances from the shared state.
    let random_selection: Vec<ArtistPerformance> = state
        .all_performances
        .choose_multiple(&mut rng, count)
        .cloned()
        .collect();

    if !random_selection.is_empty() {
        (StatusCode::OK, Json(random_selection)).into_response()
    } else {
        (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "No performances found."})),
        )
            .into_response()
    }
}

/// API handler for downloading the complete list of performances.
async fn all_bands_handler(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let headers = [
        (header::CONTENT_TYPE, "application/json".to_string()),
        // This header suggests that the browser should download the file.
        (
            header::CONTENT_DISPOSITION,
            "attachment; filename=\"all_bands.json\"".to_string(),
        ),
    ];

    (headers, Json(state.all_performances.clone()))
}

// --- Static HTML Content ---
const HTML_PAGE: &str = r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Festival Band Randomizer & Search</title>
    <script src="https://cdn.tailwindcss.com"></script>
</head>
<body class="bg-gray-900 text-gray-200 font-sans">
    <div class="container mx-auto p-4 md:p-8 max-w-4xl">

        <header class="text-center mb-10">
            <h1 class="text-4xl md:text-5xl font-bold text-white mb-2">Festival Artist Explorer</h1>
            <p class="text-lg text-gray-400">Discover artists from Pinkpop & Lowlands (2008-2019)</p>
        </header>

        <main class="space-y-12">

            <!-- Randomizer Section -->
            <section id="randomizer">
                <h2 class="text-2xl font-semibold mb-4 text-purple-400 border-b-2 border-gray-700 pb-2">Get Random Bands</h2>
                <div class="bg-gray-800 rounded-xl shadow-lg p-6 md:p-8">
                    <form id="band-form" class="flex flex-col sm:flex-row items-center gap-4">
                        <label for="count-select" class="text-lg font-medium whitespace-nowrap">How many bands?</label>
                        <select id="count-select" class="flex-grow bg-gray-700 border border-gray-600 text-white rounded-md p-2 focus:ring-2 focus:ring-purple-500 focus:outline-none">
                            <option value="1">1</option>
                            <option value="2">2</option>
                            <option value="3" selected>3</option>
                            <option value="4">4</option>
                            <option value="5">5</option>
                        </select>
                        <button type="submit" id="random-btn" class="w-full sm:w-auto bg-purple-600 hover:bg-purple-700 text-white font-bold py-2 px-6 rounded-md transition duration-300 ease-in-out transform hover:scale-105">
                            Discover
                        </button>
                    </form>
                </div>
                <div id="random-results-container" class="mt-6 space-y-4"></div>
            </section>

            <!-- Search Section -->
            <section id="search">
                 <h2 class="text-2xl font-semibold mb-4 text-teal-400 border-b-2 border-gray-700 pb-2">Search for an Artist</h2>
                 <div class="bg-gray-800 rounded-xl shadow-lg p-6 md:p-8">
                    <input type="text" id="search-input" placeholder="Type an artist name (e.g., 'kaiser')..." class="w-full bg-gray-700 border border-gray-600 text-white rounded-md p-3 text-lg focus:ring-2 focus:ring-teal-500 focus:outline-none">
                 </div>
                 <div id="search-results-container" class="mt-6 space-y-4"></div>
            </section>

             <!-- Download Section -->
            <section id="download" class="text-center mt-16">
                 <a href="/api/all-bands" class="bg-gray-700 hover:bg-gray-600 text-gray-300 font-bold py-3 px-6 rounded-md transition duration-300 ease-in-out">
                    Download Full List (.json)
                </a>
            </section>

        </main>
    </div>

    <script>
        // --- Globals ---
        const randomForm = document.getElementById('band-form');
        const countSelect = document.getElementById('count-select');
        const randomBtn = document.getElementById('random-btn');
        const randomResultsContainer = document.getElementById('random-results-container');

        const searchInput = document.getElementById('search-input');
        const searchResultsContainer = document.getElementById('search-results-container');

        // This will hold all the performance data once fetched
        let allPerformances = [];
        let isDataFetched = false;

        // --- Event Listeners ---

        // Listener for the randomizer form
        randomForm.addEventListener('submit', async (event) => {
            event.preventDefault();

            randomBtn.disabled = true;
            randomBtn.textContent = 'Loading...';
            randomResultsContainer.innerHTML = '<p class="text-center text-gray-400">Fetching artists...</p>';

            try {
                const count = countSelect.value;
                const response = await fetch(`/api/random-bands?count=${count}`);
                if (!response.ok) throw new Error(`HTTP error! status: ${response.status}`);
                const performances = await response.json();

                displayPerformances(performances, randomResultsContainer, 'purple');

            } catch (error) {
                console.error("Failed to fetch random bands:", error);
                randomResultsContainer.innerHTML = '<p class="text-center text-red-400">Failed to load bands. Please try again.</p>';
            } finally {
                randomBtn.disabled = false;
                randomBtn.textContent = 'Discover';
            }
        });

        // Listener for the search input field
        searchInput.addEventListener('input', () => {
            handleSearch();
        });

        // Fetch all data when the search box is first focused to improve perceived performance
        searchInput.addEventListener('focus', () => {
            if (!isDataFetched) {
                fetchAllBandData();
            }
        }, { once: true });


        // --- Core Functions ---

        async function fetchAllBandData() {
            try {
                const response = await fetch('/api/all-bands');
                if(!response.ok) throw new Error('Network response was not ok');
                allPerformances = await response.json();
                isDataFetched = true;
                console.log(`Fetched ${allPerformances.length} total performances.`);
            } catch (error) {
                console.error("Could not fetch all band data:", error);
                searchResultsContainer.innerHTML = `<p class="text-center text-red-500">Could not load search data. Please refresh.</p>`;
            }
        }

        function handleSearch() {
            if (!isDataFetched) {
                searchResultsContainer.innerHTML = `<p class="text-center text-gray-500">Start typing to search...</p>`;
                return;
            }

            const query = searchInput.value.trim().toLowerCase();

            if (query.length < 2) {
                searchResultsContainer.innerHTML = ''; // Clear results if query is too short
                return;
            }

            const filtered = allPerformances.filter(perf =>
                perf.name.toLowerCase().includes(query)
            );

            displayPerformances(filtered, searchResultsContainer, 'teal');

            if(filtered.length === 0) {
                 searchResultsContainer.innerHTML = `<p class="text-center text-gray-400">No matches found for "${searchInput.value}".</p>`;
            }
        }

        // --- Utility Functions ---

        function displayPerformances(performances, container, color) {
            container.innerHTML = ''; // Clear previous results
            performances.forEach(perf => {
                const card = createPerformanceCard(perf, color);
                container.appendChild(card);
            });
        }

        function createPerformanceCard(perf, color) {
            const card = document.createElement('div');
            card.className = `bg-gray-800 p-5 rounded-lg shadow-md transition transform hover:scale-[1.02] duration-300 border-l-4 border-${color}-500`;
            card.innerHTML = `
                <h3 class="text-2xl font-bold text-${color}-400">${perf.name}</h3>
                <p class="text-gray-400 mt-1">
                    Played at <span class="font-semibold text-gray-300">${perf.festival}</span> in <span class="font-semibold text-gray-300">${perf.year}</span>
                </p>
            `;
            return card;
        }

    </script>
</body>
</html>
"#;
