int main(void) {
  int a = 0; 
  do {
    a = a+1;

    if (a == 12) {
      break;
    } else {
      continue;
    }

  } while (a < 10);
  return a;
}
