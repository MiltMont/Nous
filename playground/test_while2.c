int main(void) {
  int a = 1; 
  int b = 2;
  while (a < 3) {
    while(b < 3) {
      continue;
    }
    if (a == 2) {
      break;
    }
  }
}
