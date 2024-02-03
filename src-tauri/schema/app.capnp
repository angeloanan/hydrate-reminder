@0xedf830bbfaf345b2;

struct AppState {
  # App "save" file

  version @0: UInt16;
  # Should be incremeted every time the file structure changes

  hasOnboarded @1: Bool = false;

  drinkHistory @2: List(DrinkPoint);
}

struct DrinkPoint {
  # A point in time when a drink was consumed

  timestamp @0: Int64;
  # The time when the drink was consumed

  amount @1: Float64;
  # The drink that was consumed
}
