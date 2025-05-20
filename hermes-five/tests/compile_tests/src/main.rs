use hermes_five::pause;

#[hermes_five::runtime]
async fn main() {
    pause!(500);
}